-- Sessão de login. O id é o SHA-256 do token que vai no cookie; o token em si
-- nunca é guardado.
create table sessions (
    id         bytea primary key,
    user_id    uuid not null references users (id) on delete cascade,
    created_at timestamptz not null default now(),
    expires_at timestamptz not null
);

create index sessions_user_id_idx on sessions (user_id);
