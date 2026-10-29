-- Excluir conta e excluir comunidade (decisões em "Consistência do banco" no
-- docs/decisions.md). As duas são agendadas para 7 dias e executadas pelo
-- agendador da API. A data agendada no futuro fica com a API (o `now()` de
-- quem grava não é o de quem confere, ver "Fica fora do banco"); a forma fica
-- aqui.
--
-- * Conta: `users.deletion_due_at` marca a exclusão pedida (entrar de novo
--   cancela) e `users.deleted_at`, a conta já anonimizada: nome vazio (o
--   marcador que o front mostra como "Conta excluída", também no apelido de
--   cada comunidade), sem avatar, sem email, sem login, sem inscrição de push
--   e inativa em todas as comunidades. A linha fica, para os torneios jogados
--   continuarem inteiros, e não volta a ser uma conta. Quem é owner não agenda
--   (transfere a posse ou exclui a comunidade antes), e quem tem a exclusão
--   agendada não vira owner.
-- * Comunidade: `tenants.deletion_due_at` e `deletion_requested_by`, sempre o
--   owner atual. Não agenda com torneio em andamento, e torneio não começa com
--   a exclusão agendada. A execução apaga o tenant, e a cascata leva tudo o
--   que está dentro.
--
-- Colunas novas e nulas, e nenhuma conta nem apelido vazios hoje (os checks
-- antigos não deixavam): nenhum dado existente viola as regras abaixo.

-- 1. Conta

alter table users
    -- Quando a exclusão pedida é executada (nulo: nenhuma agendada).
    add column deletion_due_at timestamptz,
    -- Quando a conta foi anonimizada (nulo: conta ativa).
    add column deleted_at timestamptz,
    drop constraint users_display_name_check,
    add constraint users_display_name_check check ((display_name = '') = (deleted_at is not null)),
    add constraint users_deleted_check
        check (deleted_at is null or (email is null and avatar_url is null and deletion_due_at is null));

-- O agendador procura as vencidas a cada passada.
create index users_deletion_due_idx on users (deletion_due_at) where deletion_due_at is not null;

alter table tenant_members
    drop constraint tenant_members_nickname_check,
    -- Vazio só na conta excluída (tenant_members_deleted_user, abaixo).
    add constraint tenant_members_nickname_check check (length(nickname) <= 32);

-- Apelido vazio é o da conta excluída, e só dela; a conta excluída não é
-- membro ativo. No commit, relendo a linha: a anonimização tira das
-- comunidades e apaga os apelidos antes de marcar a conta.
create function tenant_members_deleted_user() returns trigger
language plpgsql as $$
declare
    member record;
begin
    select u.deleted_at is not null as deleted, m.nickname, m.left_at
      into member
      from tenant_members m
      join users u on u.id = m.user_id
     where m.tenant_id = new.tenant_id and m.user_id = new.user_id;
    if not found then
        return null;
    end if;
    if (member.nickname = '') <> member.deleted then
        raise exception 'apelido vazio é só de conta excluída (membro %)', new.user_id
            using errcode = 'check_violation', constraint = 'tenant_members_nickname_deleted';
    end if;
    if member.deleted and member.left_at is null then
        raise exception 'conta excluída % não é membro ativo', new.user_id
            using errcode = 'check_violation', constraint = 'tenant_members_user_deleted';
    end if;
    return null;
end $$;

create constraint trigger tenant_members_deleted_user
    after insert or update of nickname, left_at on tenant_members
    deferrable initially deferred
    for each row execute function tenant_members_deleted_user();

-- Conta excluída: sem login, sem inscrição de push, inativa e sem apelido em
-- todas as comunidades. No commit, como a de cima.
create function users_deleted_clean() returns trigger
language plpgsql as $$
begin
    if not exists (select 1 from users where id = new.id and deleted_at is not null) then
        return null;
    end if;
    if exists (select 1 from user_identities where user_id = new.id)
       or exists (select 1 from password_credentials where user_id = new.id)
       or exists (select 1 from push_subscriptions where user_id = new.id)
    then
        raise exception 'conta excluída % com login ou inscrição de push', new.id
            using errcode = 'check_violation', constraint = 'users_deleted_logins';
    end if;
    if exists (select 1 from tenant_members
               where user_id = new.id and (left_at is null or nickname <> ''))
    then
        raise exception 'conta excluída % ainda é membro ou tem apelido', new.id
            using errcode = 'check_violation', constraint = 'users_deleted_members';
    end if;
    return null;
end $$;

create constraint trigger users_deleted_clean
    after update of deleted_at on users
    deferrable initially deferred
    for each row when (new.deleted_at is not null)
    execute function users_deleted_clean();

-- A anonimização não se desfaz.
create function users_deleted_final() returns trigger
language plpgsql as $$
begin
    raise exception 'conta % já foi excluída', old.id
        using errcode = 'check_violation', constraint = 'users_deleted_final';
end $$;

create trigger users_deleted_final
    before update of deleted_at on users
    for each row when (old.deleted_at is not null and new.deleted_at is distinct from old.deleted_at)
    execute function users_deleted_final();

-- Conta excluída não ganha login nem inscrição de push (`tg_argv[0]`: o nome
-- da regra).
create function refuse_deleted_user() returns trigger
language plpgsql as $$
begin
    if exists (select 1 from users where id = new.user_id and deleted_at is not null) then
        raise exception 'conta % excluída', new.user_id
            using errcode = 'check_violation', constraint = tg_argv[0];
    end if;
    return new;
end $$;

create trigger user_identities_user_deleted
    before insert or update on user_identities
    for each row execute function refuse_deleted_user('user_identities_user_deleted');
create trigger password_credentials_user_deleted
    before insert or update on password_credentials
    for each row execute function refuse_deleted_user('password_credentials_user_deleted');
create trigger push_subscriptions_user_deleted
    before insert or update on push_subscriptions
    for each row execute function refuse_deleted_user('push_subscriptions_user_deleted');

-- Owner não agenda a exclusão da conta, e quem a agendou não vira owner. As
-- duas escritas travam a linha do usuário (agendar a atualiza; criar tenant e
-- transferir a posse fazem `for update` nela), então não passam juntas.
create function users_deletion_owner() returns trigger
language plpgsql as $$
begin
    if exists (select 1 from tenant_members where user_id = new.id and role = 'owner') then
        raise exception 'usuário % é owner e não agenda a exclusão da conta', new.id
            using errcode = 'check_violation', constraint = 'users_deletion_owner';
    end if;
    return new;
end $$;

create trigger users_deletion_owner
    before update of deletion_due_at on users
    for each row when (new.deletion_due_at is not null)
    execute function users_deletion_owner();

create function tenant_members_owner_deleting() returns trigger
language plpgsql as $$
begin
    if exists (select 1 from users where id = new.user_id and deletion_due_at is not null) then
        raise exception 'usuário % tem a exclusão da conta agendada e não vira owner', new.user_id
            using errcode = 'check_violation', constraint = 'tenant_members_owner_deleting';
    end if;
    return new;
end $$;

create trigger tenant_members_owner_deleting
    before insert or update of role on tenant_members
    for each row when (new.role = 'owner')
    execute function tenant_members_owner_deleting();

-- 2. Comunidade

alter table tenants
    -- Quando a exclusão pedida é executada (nulo: nenhuma agendada). Até lá,
    -- a comunidade fica só leitura.
    add column deletion_due_at timestamptz,
    -- Quem pediu, sempre o owner (nulo: nenhuma exclusão agendada).
    add column deletion_requested_by bigint,
    -- Só a cascata do próprio tenant apaga a linha do owner.
    add constraint tenants_deletion_requested_by_fkey foreign key (id, deletion_requested_by)
        references tenant_members (tenant_id, user_id),
    add constraint tenants_deletion_check
        check ((deletion_due_at is null) = (deletion_requested_by is null));

create index tenants_deletion_due_idx on tenants (deletion_due_at) where deletion_due_at is not null;

-- Quem pediu é o owner, no commit (transferir a posse rebaixa um e promove o
-- outro na mesma transação).
create function check_tenant_deletion_owner(tenant bigint) returns void
language plpgsql as $$
begin
    if exists (select 1 from tenants t
               where t.id = tenant and t.deletion_requested_by is not null
                 and not exists (select 1 from tenant_members m
                                 where m.tenant_id = t.id and m.user_id = t.deletion_requested_by
                                   and m.role = 'owner'))
    then
        raise exception 'exclusão do tenant % pedida por quem não é o owner', tenant
            using errcode = 'check_violation', constraint = 'tenants_deletion_by_owner';
    end if;
end $$;

create function tenants_deletion_by_owner() returns trigger
language plpgsql as $$
begin
    perform check_tenant_deletion_owner(new.id);
    return null;
end $$;

create function tenant_members_deletion_by_owner() returns trigger
language plpgsql as $$
begin
    perform check_tenant_deletion_owner(old.tenant_id);
    return null;
end $$;

create constraint trigger tenants_deletion_by_owner
    after insert or update of deletion_requested_by on tenants
    deferrable initially deferred
    for each row when (new.deletion_requested_by is not null)
    execute function tenants_deletion_by_owner();

create constraint trigger tenant_members_deletion_by_owner
    after update of role on tenant_members
    deferrable initially deferred
    for each row when (old.role = 'owner' and new.role <> 'owner')
    execute function tenant_members_deletion_by_owner();

-- Torneio em andamento e exclusão agendada não convivem: só leitura
-- congelaria a partida no meio. Iniciar trava a linha do tenant (`for share`)
-- e agendar a atualiza, então as duas não passam juntas.
create function tenants_deletion_in_progress() returns trigger
language plpgsql as $$
begin
    if exists (select 1 from tournaments where tenant_id = new.id and status = 'in_progress') then
        raise exception 'tenant % tem torneio em andamento e não agenda a exclusão', new.id
            using errcode = 'check_violation', constraint = 'tenants_deletion_in_progress';
    end if;
    return new;
end $$;

create trigger tenants_deletion_in_progress
    before insert or update of deletion_due_at on tenants
    for each row when (new.deletion_due_at is not null)
    execute function tenants_deletion_in_progress();

create function tournaments_tenant_deleting() returns trigger
language plpgsql as $$
declare
    due timestamptz;
begin
    select deletion_due_at into due from tenants where id = new.tenant_id for share;
    if due is not null then
        raise exception 'tenant % tem a exclusão agendada, e o torneio % não começa', new.tenant_id, new.id
            using errcode = 'check_violation', constraint = 'tournaments_tenant_deleting';
    end if;
    return new;
end $$;

create trigger tournaments_tenant_deleting
    before insert or update of status on tournaments
    for each row when (new.status = 'in_progress')
    execute function tournaments_tenant_deleting();
