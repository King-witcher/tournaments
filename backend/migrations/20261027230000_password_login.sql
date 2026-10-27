-- Login com username e senha: mais um provedor em user_identities, com o
-- username (como digitado) em provider_user_id e o hash argon2 da senha.
alter table user_identities
    drop constraint user_identities_provider_check,
    add constraint user_identities_provider_check check (provider in ('discord', 'password')),
    add column password_hash text,
    add constraint user_identities_password_hash_check
        check ((provider = 'password') = (password_hash is not null));

-- Username único sem diferenciar maiúsculas: "Rapha" e "rapha" são o mesmo
-- (o formato só aceita ASCII, então lower() basta).
create unique index user_identities_username_idx
    on user_identities (lower(provider_user_id))
    where provider = 'password';

-- No máximo um login de cada provedor por usuário. Substitui o índice por
-- user_id, que é o prefixo deste.
drop index user_identities_user_id_idx;
create unique index user_identities_user_provider_idx on user_identities (user_id, provider);

-- Senhas erradas recentes, para o limite de tentativas. A chave é o username
-- em minúsculas, exista a conta ou não. Linhas fora da janela são apagadas a
-- cada erro novo.
create table password_failures (
    id        bigint generated always as identity primary key,
    username  text not null,
    failed_at timestamptz not null default now()
);

create index password_failures_username_idx on password_failures (username, failed_at);
create index password_failures_failed_at_idx on password_failures (failed_at);
