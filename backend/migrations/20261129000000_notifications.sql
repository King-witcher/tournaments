-- Notificações ("Notificações" em docs/decisions.md): os avisos para pessoas
-- (o `Notice` do outbox), guardados um por destinatário quando o despachante
-- os publica, para o sininho do cabeçalho mostrar os que já passaram (o toast
-- some sozinho, e com o site fechado só chega o Web Push) e quantos a pessoa
-- ainda não viu. Saem com 30 dias (agendador) e com a conta excluída.
--
-- Dados existentes: tabela nova, nada a corrigir; o check do outbox só ganha
-- um tipo (todo evento que passava passa).

-- `unread`: a pessoa viu as notificações numa aba, e as outras recebem quantas
-- faltam.
create or replace function outbox_event_valid(event jsonb) returns boolean
language sql immutable as $$
    select coalesce(
        jsonb_typeof(event) = 'object'
        and event ->> 'kind' in
            ('tenant', 'tournament', 'me', 'access_lost', 'sessions_ended', 'notice',
             'announce', 'unread'),
        false)
$$;

-- {"code": "match.ready", "params": {...}, "path": "/quake-br/copa/..."}: a
-- forma do `Notice` (outbox/event.rs). O código e os parâmetros de cada aviso
-- ficam com a API, que pula, com log, o que ela não entende mais.
create function notification_notice_valid(notice jsonb) returns boolean
language sql immutable as $$
    select coalesce(
        jsonb_typeof(notice) = 'object'
        and jsonb_typeof(notice -> 'code') = 'string'
        and jsonb_typeof(notice -> 'params') = 'object'
        and jsonb_typeof(notice -> 'path') = 'string'
        and notice ->> 'path' like '/%',
        false)
$$;

create table notifications (
    id         bigint generated always as identity primary key,
    user_id    bigint not null references users (id) on delete cascade,
    -- O aviso como o front o recebe pelo WebSocket: código, parâmetros e o
    -- caminho no site, sem o idioma.
    notice     jsonb not null
               constraint notifications_notice_valid check (notification_notice_valid(notice)),
    created_at timestamptz not null default now(),
    -- Quando a pessoa abriu o sininho com ela na lista; nulo enquanto não viu.
    read_at    timestamptz
);

-- O sininho: as mais recentes de quem vê.
create index notifications_by_user on notifications (user_id, id desc);
-- Quantas faltam ver, a cada conexão do WebSocket.
create index notifications_unread on notifications (user_id) where read_at is null;
-- A limpeza das que passaram do prazo, a cada passada do agendador.
create index notifications_created on notifications (created_at);
