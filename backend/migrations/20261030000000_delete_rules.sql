-- O que cada exclusão faz, e a posse do tenant (decisões em "Consistência do
-- banco" no docs/decisions.md):
--
-- * Usuário: recusado se for owner de algum tenant ou se tiver participado de
--   torneio iniciado (em andamento ou encerrado), para a chave não perder um
--   lado. Sessões, logins e tentativas de senha vão junto. A autoria (quem
--   inscreveu, quem marcou o check-in, quem enviou, confirmou ou contestou um
--   resultado, quem agiu no histórico) fica nula; os convites que ele criou
--   saem com a participação dele no tenant. "Excluir conta" vai anonimizar o
--   usuário em vez de apagar.
-- * Tenant: apaga tudo o que está dentro (membros, convites, torneios e o que
--   eles têm).
-- * Torneio: apaga tudo o que está dentro, inclusive resultados. A API só
--   exclui rascunho; o banco aceita a cascata de qualquer um.
-- * Inscrito e fase de torneio iniciado não saem avulsos, só com o torneio ou
--   o tenant: na cascata deles o pai já foi apagado quando o filho sai. As FKs
--   de resultados e histórico para inscritos e fases são `no action` (e não
--   `restrict`), conferidas no fim do comando: a cascata do torneio apaga os
--   resultados antes da conferência.
--
-- Recusa de exclusão por trigger sai com o código de FK (23503), como a das
-- FKs; regra de estado, com o de check (23514). As duas levam o nome da regra
-- em `constraint`.

-- 1. Autoria dentro do tenant: quem inscreveu, quem marcou o check-in e quem
-- criou o convite são membros do mesmo tenant (FK composta, como o resto).
-- Inscrição e check-in guardam o fato sem o autor quando ele sai do tenant; o
-- convite é uma autorização dada pelo membro e sai com ele.

-- Dados antigos: inscrição ou check-in marcados por quem não é membro do
-- tenant ficam sem autor.
update participants p set added_by = null
where added_by is not null
  and not exists (select 1 from tenant_members m
                  where m.tenant_id = p.tenant_id and m.user_id = p.added_by);
update participants p set checked_in_by = null
where checked_in_by is not null
  and not exists (select 1 from tenant_members m
                  where m.tenant_id = p.tenant_id and m.user_id = p.checked_in_by);

alter table participants
    drop constraint participants_added_by_fkey,
    drop constraint participants_checked_in_by_fkey,
    add constraint participants_added_by_fkey foreign key (tenant_id, added_by)
        references tenant_members (tenant_id, user_id) on delete set null (added_by),
    add constraint participants_checked_in_by_fkey foreign key (tenant_id, checked_in_by)
        references tenant_members (tenant_id, user_id) on delete set null (checked_in_by);

create index participants_added_by_idx on participants (tenant_id, added_by)
    where added_by is not null;
create index participants_checked_in_by_idx on participants (tenant_id, checked_in_by)
    where checked_in_by is not null;

-- 2. Exatamente um owner por tenant, e ele é membro (a posse é uma linha de
-- tenant_members). "No máximo um" é o índice único parcial
-- tenant_members_one_owner_idx, conferido na hora; "pelo menos um" é
-- conferido no commit, para a transferência de posse caber numa transação:
-- primeiro o owner atual deixa de ser owner, depois o novo passa a ser (na
-- ordem inversa, o índice recusa na hora).

-- Dados antigos: tenant com membros e sem owner promove o organizer mais
-- antigo (ou, sem organizer, o membro mais antigo). Tenant sem nenhum membro
-- não tem quem promover e fica como está (a API nunca cria um assim).
update tenant_members m set role = 'owner'
from (select distinct on (tenant_id) tenant_id, user_id
      from tenant_members
      where tenant_id not in (select tenant_id from tenant_members where role = 'owner')
      order by tenant_id, role = 'organizer' desc, joined_at, user_id) heir
where m.tenant_id = heir.tenant_id and m.user_id = heir.user_id;

create function check_tenant_owner(tenant bigint) returns void
language plpgsql as $$
begin
    if exists (select 1 from tenants where id = tenant)
       and not exists (select 1 from tenant_members where tenant_id = tenant and role = 'owner')
    then
        raise exception 'tenant % sem owner', tenant
            using errcode = 'check_violation', constraint = 'tenant_members_owner_required';
    end if;
end $$;

create function tenants_owner_required() returns trigger
language plpgsql as $$
begin
    perform check_tenant_owner(new.id);
    return null;
end $$;

create function tenant_members_owner_required() returns trigger
language plpgsql as $$
begin
    perform check_tenant_owner(old.tenant_id);
    return null;
end $$;

-- Tenant novo nasce com o owner na mesma transação.
create constraint trigger tenants_owner_required
    after insert on tenants
    deferrable initially deferred
    for each row execute function tenants_owner_required();

-- O owner deixou de ser owner (ou de ser membro): outro precisa ser até o
-- commit. Apagar o tenant inteiro passa (ele já não existe no commit).
create constraint trigger tenant_members_owner_required
    after update or delete on tenant_members
    deferrable initially deferred
    for each row when (old.role = 'owner')
    execute function tenant_members_owner_required();

-- Convite criado por quem não é membro do tenant passa para o owner; sem
-- owner (tenant sem membros), sai.
update tenant_invites i set created_by = o.user_id
from tenant_members o
where o.tenant_id = i.tenant_id and o.role = 'owner'
  and not exists (select 1 from tenant_members m
                  where m.tenant_id = i.tenant_id and m.user_id = i.created_by);
delete from tenant_invites i
where not exists (select 1 from tenant_members m
                  where m.tenant_id = i.tenant_id and m.user_id = i.created_by);

alter table tenant_invites
    drop constraint tenant_invites_created_by_fkey,
    add constraint tenant_invites_created_by_fkey foreign key (tenant_id, created_by)
        references tenant_members (tenant_id, user_id) on delete cascade;

create index tenant_invites_created_by_idx on tenant_invites (tenant_id, created_by);

-- 3. Torneio iniciado: em andamento ou encerrado, num tenant que ainda
-- existe. Falso também quando o torneio ou o tenant já foram apagados nesta
-- transação, que é como a cascata deles se distingue da remoção avulsa.
create function tournament_started(tournament bigint) returns boolean
language sql stable as $$
    select exists (
        select 1
        from tournaments t
        join tenants n on n.id = t.tenant_id
        where t.id = tournament and t.status in ('in_progress', 'finished'))
$$;

-- 4. Usuário: owner de tenant ou participante de torneio iniciado não sai.
create function users_delete_guard() returns trigger
language plpgsql as $$
begin
    if exists (select 1 from tenant_members where user_id = old.id and role = 'owner') then
        raise exception 'usuário % é owner de um tenant', old.id
            using errcode = 'foreign_key_violation', constraint = 'users_owner_delete';
    end if;
    if exists (select 1 from participants p
               where p.user_id = old.id and tournament_started(p.tournament_id)) then
        raise exception 'usuário % participou de torneio iniciado', old.id
            using errcode = 'foreign_key_violation', constraint = 'users_participant_delete';
    end if;
    return old;
end $$;

create trigger users_delete_guard
    before delete on users
    for each row execute function users_delete_guard();

-- 5. Inscritos de torneio iniciado ficam como estão: ninguém entra nem sai, e
-- seed e check-in não mudam (a chave sai deles). Autoria e convite podem
-- ficar nulos (o autor saiu do tenant). Torneio e membro de um inscrito
-- nunca mudam.
create function participants_frozen_after_start() returns trigger
language plpgsql as $$
begin
    if tg_op = 'DELETE' then
        if tournament_started(old.tournament_id) then
            raise exception 'inscrito % de torneio iniciado', old.id
                using errcode = 'foreign_key_violation', constraint = 'participants_started_delete';
        end if;
        return old;
    end if;
    if tg_op = 'UPDATE'
       and (new.tenant_id, new.tournament_id, new.user_id)
           is distinct from (old.tenant_id, old.tournament_id, old.user_id)
    then
        raise exception 'torneio e membro do inscrito % não mudam', old.id
            using errcode = 'check_violation', constraint = 'participants_identity_immutable';
    end if;
    if tournament_started(new.tournament_id)
       and (tg_op = 'INSERT'
            or (new.seed, new.checked_in_at) is distinct from (old.seed, old.checked_in_at))
    then
        raise exception 'torneio % já começou: inscritos, seeds e check-ins congelados',
                new.tournament_id
            using errcode = 'check_violation', constraint = 'participants_started_frozen';
    end if;
    return new;
end $$;

create trigger participants_frozen_after_start
    before insert or update or delete on participants
    for each row execute function participants_frozen_after_start();

-- 6. Fases de torneio iniciado também ficam como estão (a chave sai da
-- configuração delas).
create function tournament_stages_frozen_after_start() returns trigger
language plpgsql as $$
begin
    if tg_op = 'DELETE' then
        if tournament_started(old.tournament_id) then
            raise exception 'fase % de torneio iniciado', old.id
                using errcode = 'foreign_key_violation', constraint = 'tournament_stages_started_delete';
        end if;
        return old;
    end if;
    if tournament_started(new.tournament_id)
       or (tg_op = 'UPDATE' and tournament_started(old.tournament_id))
    then
        raise exception 'torneio % já começou: fases congeladas', new.tournament_id
            using errcode = 'check_violation', constraint = 'tournament_stages_started_frozen';
    end if;
    return new;
end $$;

create trigger tournament_stages_frozen_after_start
    before insert or update or delete on tournament_stages
    for each row execute function tournament_stages_frozen_after_start();

-- 7. Tentativas de senha saem com o login (inclusive na exclusão do usuário,
-- pela cascata) e quando o username muda: a chave delas é o username em
-- minúsculas, que deixa de existir.
create function user_identities_clear_failures() returns trigger
language plpgsql as $$
begin
    if tg_op = 'UPDATE' and lower(new.provider_user_id) = lower(old.provider_user_id) then
        return null;
    end if;
    delete from password_failures where username = lower(old.provider_user_id);
    return null;
end $$;

create trigger user_identities_clear_failures
    after delete or update of provider_user_id on user_identities
    for each row when (old.provider = 'password')
    execute function user_identities_clear_failures();
