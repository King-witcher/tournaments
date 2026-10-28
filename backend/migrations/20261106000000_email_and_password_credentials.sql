-- Email da conta e username e senha numa tabela própria ("Email" e
-- "Autenticação" em docs/decisions.md).

-- 1. Email da conta: contato e recuperação de senha, não um login. Só entra
--    confirmado (pelo link do cadastro ou da troca), na forma canônica que a
--    API grava (minúsculo, sem o "+etiqueta" e, no Gmail, sem pontos), então
--    o único da coluna também barra o mesmo endereço escrito de outro jeito.
--    Nulo: conta sem email (só Discord, ou com senha de antes do email ser
--    exigido no cadastro).
alter table users
    add column email text,
    add constraint users_email_key unique (email),
    add constraint users_email_format check (
        length(email) <= 254
        and email ~ '^[a-z0-9._-]+@[a-z0-9-]+(\.[a-z0-9-]+)+$'
        and split_part(email, '@', 2) <> 'googlemail.com'
        and (split_part(email, '@', 2) <> 'gmail.com'
             or position('.' in split_part(email, '@', 1)) = 0)
    );

-- 2. Username e senha saem de user_identities, que fica só com provedores
--    externos (Discord; Google e Steam no futuro), para o hash da senha não
--    conviver com ids de OAuth. No máximo um por conta (a chave é o user_id).
create table password_credentials (
    user_id       bigint primary key references users (id) on delete cascade,
    -- Como foi digitado; único sem diferenciar maiúsculas (índice abaixo).
    username      text not null
        constraint password_credentials_username_format
        check (username ~ '^[A-Za-z0-9_.-]{3,20}$'),
    password_hash text not null
        constraint password_credentials_password_hash_format
        check (password_hash like '$argon2id$%'),
    created_at    timestamptz not null default now()
);

-- O formato só aceita ASCII, então lower() basta.
create unique index password_credentials_username_idx on password_credentials (lower(username));

-- Os logins com senha que existem passam como estão (os checks de formato já
-- valiam em user_identities).
insert into password_credentials (user_id, username, password_hash, created_at)
select user_id, provider_user_id, password_hash, created_at
from user_identities
where provider = 'password';

delete from user_identities where provider = 'password';

drop index user_identities_username_idx;
alter table user_identities
    drop constraint user_identities_password_hash_check,
    drop constraint user_identities_username_format,
    drop constraint user_identities_password_hash_format,
    drop column password_hash,
    drop constraint user_identities_provider_check,
    add constraint user_identities_provider_check check (provider in ('discord'));
