-- Código curto de convite (docs/decisions/tenants.md, "Convites"): no aplicativo
-- instalado (PWA) não há barra de endereço, e a pessoa precisa conseguir digitar
-- o convite. Novos e girados passam a ter 8 caracteres, em minúsculas e dígitos
-- sem ambíguos (31^8); quem gera é a API (`token::random_invite_code`).
--
-- Dados existentes: os tokens de 22 caracteres continuam valendo como estão
-- (há links já compartilhados de torneios em andamento), por isso o `check`
-- aceita os dois formatos, e nenhuma linha é reescrita.

alter table tenants
    drop constraint tenants_invite_token_format,
    add constraint tenants_invite_token_format
        check (invite_token ~ '^([1-9A-HJ-NP-Za-km-z]{22}|[2-9a-hjkmnp-z]{8})$');

alter table tournaments
    drop constraint tournaments_invite_token_format,
    add constraint tournaments_invite_token_format
        check (invite_token ~ '^([1-9A-HJ-NP-Za-km-z]{22}|[2-9a-hjkmnp-z]{8})$');

-- O `unique` de cada coluna não cobre a outra tabela, e `GET /invites/{token}`
-- olha as duas: um código repetido levaria a pessoa ao lugar errado. A API
-- procura um código livre antes de gravar (`tenants::free_invite_code`); isto
-- garante mesmo com duas gravações ao mesmo tempo, que a trava (presa até o fim
-- da transação) põe em fila antes de conferir a outra tabela.
create function invite_token_not_in_other_table() returns trigger
language plpgsql as $$
begin
    perform pg_advisory_xact_lock(hashtextextended(new.invite_token, 0));
    if (tg_table_name = 'tenants'
        and exists (select 1 from tournaments where invite_token = new.invite_token))
       or (tg_table_name = 'tournaments'
           and exists (select 1 from tenants where invite_token = new.invite_token))
    then
        raise exception 'código de convite % já existe na outra tabela', new.invite_token
            using errcode = 'unique_violation',
                  constraint = tg_table_name || '_invite_token_taken';
    end if;
    return new;
end $$;

create trigger tenants_invite_token_taken
    before insert or update of invite_token on tenants
    for each row execute function invite_token_not_in_other_table();

create trigger tournaments_invite_token_taken
    before insert or update of invite_token on tournaments
    for each row execute function invite_token_not_in_other_table();
