-- Avisos da comunidade no Discord por webhook (decisões em "Notificações" no
-- docs/decisions.md): a configuração de cada comunidade, a fila de mensagens
-- (o Discord limita o ritmo por webhook, e o despachante do outbox não pode
-- esperar por ele) e o evento `announce` do outbox, para os avisos que só
-- vão para o canal (torneio iniciado, encerrado, série decidida).

create or replace function outbox_event_valid(event jsonb) returns boolean
language sql immutable as $$
    select coalesce(
        jsonb_typeof(event) = 'object'
        and event ->> 'kind' in
            ('tenant', 'tournament', 'me', 'access_lost', 'sessions_ended', 'notice',
             'announce'),
        false)
$$;

-- Códigos de aviso que a comunidade liga (`check_in.opened`, `match.ready`):
-- só a forma e a ausência de repetição. Quais códigos existem é da API, para
-- um aviso novo (o veto) entrar sem migration; o código desconhecido fica
-- guardado e nunca é enviado. Falso (nunca erro) para qualquer entrada.
create function discord_events_valid(events text[]) returns boolean
language sql immutable as $$
    select coalesce(
        cardinality(events) <= 32
        and not exists (
            select 1 from unnest(events) as e
            where e is null or e !~ '^[a-z_]+(\.[a-z_]+)+$')
        and (select count(distinct e) = cardinality(events) from unnest(events) as e),
        false)
$$;

-- O webhook de um canal da comunidade (no máximo um). O token é credencial:
-- fica em claro porque o envio precisa dele, e a API nunca o devolve nem o
-- escreve no log.
create table discord_webhooks (
    tenant_id       bigint primary key references tenants (id) on delete cascade,
    -- Os dois pedaços de https://discord.com/api/webhooks/{id}/{token}.
    webhook_id      text not null
                    constraint discord_webhooks_id_format check (webhook_id ~ '^[0-9]{1,20}$'),
    token           text not null
                    constraint discord_webhooks_token_format
                    check (token ~ '^[A-Za-z0-9_-]{20,100}$'),
    -- Os avisos ligados (códigos, como em `Notice`).
    events          text[] not null
                    constraint discord_webhooks_events_valid check (discord_events_valid(events)),
    -- Menciona (`<@id>`) quem vinculou o Discord à conta.
    mention_players boolean not null,
    -- Idioma das mensagens.
    locale          text not null
                    constraint discord_webhooks_locale_check check (locale in ('pt', 'es')),
    -- Nulo: funciona. Preenchido: o Discord respondeu 401 ou 404 (o webhook
    -- foi apagado) e a API parou de tentar até o owner colar outra URL.
    broken_at       timestamptz,
    -- Nulo: sem espera. Preenchido: o Discord pediu para esperar até lá
    -- (limite de ritmo do webhook).
    retry_at        timestamptz,
    created_at      timestamptz not null default now(),
    updated_at      timestamptz not null default now()
);

-- Mensagens esperando o envio, em ordem. `code`, `params` e `path` são os do
-- aviso (o texto sai na hora do envio, no idioma da configuração).
create table discord_messages (
    id         bigint generated always as identity primary key,
    tenant_id  bigint not null references discord_webhooks (tenant_id) on delete cascade,
    code       text not null
               constraint discord_messages_code_format check (code ~ '^[a-z_]+(\.[a-z_]+)+$'),
    params     jsonb not null
               constraint discord_messages_params_object check (jsonb_typeof(params) = 'object'),
    -- Caminho no site, sem o idioma: /{comunidade}/{torneio}/...
    path       text not null
               constraint discord_messages_path_format check (path ~ '^/[^ ]*$'),
    -- Usuários a quem a mensagem se dirige (mencionados, se ligado).
    users      bigint[] not null default '{}',
    -- Nulo: mensagem própria. Preenchido: avisos iguais ainda na fila viram
    -- uma só, juntando os `users` (a partida pronta chega uma vez por lado).
    merge_key  text,
    -- Tentativas já feitas (vale também para a que está em curso).
    attempts   smallint not null default 0
               constraint discord_messages_attempts_range check (attempts between 0 and 20),
    -- Não sai antes disso: a espera da nova tentativa e a reserva de quem está
    -- enviando agora.
    not_before timestamptz not null default now(),
    created_at timestamptz not null default now()
);

create unique index discord_messages_merge on discord_messages (tenant_id, merge_key)
    where merge_key is not null;
create index discord_messages_due on discord_messages (id) include (tenant_id, not_before);
