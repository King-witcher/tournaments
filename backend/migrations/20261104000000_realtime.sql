-- Tempo real e avisos (decisões em "Tempo real e avisos" no docs/decisions.md):
-- o outbox dos eventos de mudança, as inscrições de Web Push dos navegadores e
-- a marca do aviso "hora de iniciar" de cada torneio.

-- Forma mínima de um evento do outbox: objeto com um "kind" conhecido. O
-- resto da forma é do despachante da API, que descarta (com log) o que não
-- entende.
create function outbox_event_valid(event jsonb) returns boolean
language sql immutable as $$
    select coalesce(
        jsonb_typeof(event) = 'object'
        and event ->> 'kind' in
            ('tenant', 'tournament', 'me', 'access_lost', 'sessions_ended', 'notice'),
        false)
$$;

-- Eventos gravados na transação de quem mudou o estado. Depois do commit, o
-- despachante da API lê em ordem, publica no WebSocket, manda os Web Push e
-- apaga a linha: só sai evento de transação confirmada, e o push não se perde
-- se a API cair entre o commit e o envio.
create table outbox (
    id         bigint generated always as identity primary key,
    -- Transação que gravou: o mesmo evento repetido nela entra uma vez só.
    xact       xid8 not null default pg_current_xact_id(),
    event      jsonb not null constraint outbox_event_shape check (outbox_event_valid(event)),
    created_at timestamptz not null default now()
);

-- Um evento igual por transação. Pelo md5 do texto (o do jsonb é canônico):
-- o evento inteiro passaria do tamanho máximo de uma linha de índice com a
-- lista de inscritos de um torneio grande.
create unique index outbox_once_per_xact on outbox (xact, md5(event::text));

-- Inscrição de Web Push de um navegador. O endpoint é do navegador: se outra
-- conta se inscreve nele, a linha passa para ela.
create table push_subscriptions (
    id           bigint generated always as identity primary key,
    user_id      bigint not null references users (id) on delete cascade,
    -- URL do serviço de push do navegador.
    endpoint     text not null unique
                 constraint push_subscriptions_endpoint_format
                 check (endpoint ~ '^https://[!-~]+$' and length(endpoint) <= 2048),
    -- Chave pública ECDH do navegador (P-256 sem compressão, 65 bytes) e o
    -- segredo de autenticação (16 bytes), em base64url sem padding.
    p256dh       text not null
                 constraint push_subscriptions_p256dh_format check (p256dh ~ '^[A-Za-z0-9_-]{87}$'),
    auth         text not null
                 constraint push_subscriptions_auth_format check (auth ~ '^[A-Za-z0-9_-]{22}$'),
    -- Idioma do texto do aviso (o da página quando se inscreveu).
    locale       text not null
                 constraint push_subscriptions_locale_check check (locale in ('pt', 'es')),
    created_at   timestamptz not null default now(),
    -- Último aviso entregue (nulo: nenhum ainda).
    last_used_at timestamptz
);

-- Os avisos de um usuário saem para todas as inscrições dele.
create index push_subscriptions_user_id_idx on push_subscriptions (user_id);

alter table tournaments
    -- Quando saiu o aviso "hora de iniciar" para organizers e owner (nulo:
    -- ainda não). Vale para o início que já tinha chegado: adiar o início para
    -- depois desse momento avisa de novo.
    add column start_reminded_at timestamptz,
    add constraint tournaments_start_reminded_published
        check (start_reminded_at is null or status <> 'draft');

-- Dados existentes: os torneios publicados e não iniciados cujo início já
-- passou ficam como avisados agora, para a primeira passada do agendador não
-- avisar de uma vez os torneios esquecidos.
update tournaments
set start_reminded_at = now()
where status in ('registration', 'check_in') and starts_at <= now();
