-- Usuários são globais; o perfil em cada tenant vem depois (tenant_players).
create table users (
    id           uuid primary key default gen_random_uuid(),
    display_name text not null,
    avatar_url   text,
    locale       text not null default 'pt' check (locale in ('pt', 'es')),
    created_at   timestamptz not null default now()
);

-- Cada forma de login é uma linha. Novos provedores (senha, Steam) entram
-- ampliando o check, sem mudar a estrutura.
create table user_identities (
    id               uuid primary key default gen_random_uuid(),
    user_id          uuid not null references users (id) on delete cascade,
    provider         text not null check (provider in ('discord')),
    provider_user_id text not null,
    created_at       timestamptz not null default now(),
    unique (provider, provider_user_id)
);

create index user_identities_user_id_idx on user_identities (user_id);
