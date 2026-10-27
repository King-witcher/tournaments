-- Usuários são globais; o perfil em cada tenant fica em tenant_members.
-- O id não é segredo (ver "Ids" em docs/decisions.md): bigint sequencial.
create table users (
    id           bigint generated always as identity primary key,
    display_name text not null,
    avatar_url   text,
    locale       text not null default 'pt' check (locale in ('pt', 'es')),
    created_at   timestamptz not null default now()
);

-- Cada forma de login é uma linha. Novos provedores (senha, Steam) entram
-- ampliando o check, sem mudar a estrutura.
create table user_identities (
    provider         text not null check (provider in ('discord')),
    provider_user_id text not null,
    user_id          bigint not null references users (id) on delete cascade,
    created_at       timestamptz not null default now(),
    primary key (provider, provider_user_id)
);

create index user_identities_user_id_idx on user_identities (user_id);
