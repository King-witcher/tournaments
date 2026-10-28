-- Membro inativo (decisões em "Multi-tenant" no docs/decisions.md): sair da
-- comunidade ou ser removido não apaga a linha de tenant_members (o banco não
-- deixaria, se ele jogou torneio iniciado), marca left_at. O inativo não tem
-- acesso (a API responde como para quem é de fora), some das listas, não é
-- inscrito e não tem convite aberto; nos torneios iniciados, a inscrição dele
-- fica (é histórico). Voltar por convite reativa a mesma linha, com apelido e
-- histórico, como player.
--
-- Colunas novas e nulas: nenhum dado existente viola as regras abaixo.

alter table tenant_members
    -- Quando saiu ou foi removido (nulo: ativo).
    add column left_at timestamptz,
    -- Quem removeu (nulo: ativo, ou saiu sozinho). Sai com o autor.
    add column removed_by bigint,
    add constraint tenant_members_removed_by_fkey foreign key (tenant_id, removed_by)
        references tenant_members (tenant_id, user_id) on delete set null (removed_by),
    add constraint tenant_members_removed_by_check
        check (removed_by is null or (left_at is not null and removed_by <> user_id)),
    -- O owner transfere a posse antes de sair.
    add constraint tenant_members_owner_active check (role <> 'owner' or left_at is null);

create index tenant_members_removed_by_idx on tenant_members (tenant_id, removed_by)
    where removed_by is not null;

-- Ao sair: nenhum convite aberto criado por ele (a API revoga antes; a FK
-- composta só apagaria os convites com a linha) e nenhuma inscrição em
-- torneio que não começou (a API tira antes).
create function tenant_members_leave_guard() returns trigger
language plpgsql as $$
begin
    if exists (select 1 from tenant_invites
               where tenant_id = new.tenant_id and created_by = new.user_id
                 and revoked_at is null)
    then
        raise exception 'membro % sai com convites abertos', new.user_id
            using errcode = 'check_violation', constraint = 'tenant_members_left_invites_open';
    end if;
    if exists (select 1 from participants
               where tenant_id = new.tenant_id and user_id = new.user_id
                 and not tournament_started(tournament_id))
    then
        raise exception 'membro % sai inscrito em torneio que não começou', new.user_id
            using errcode = 'check_violation', constraint = 'tenant_members_left_registered';
    end if;
    return new;
end $$;

create trigger tenant_members_leave_guard
    before update of left_at on tenant_members
    for each row when (old.left_at is null and new.left_at is not null)
    execute function tenant_members_leave_guard();

-- Convite aberto é de membro ativo: o inativo não cria nem reabre.
create function tenant_invites_creator_active() returns trigger
language plpgsql as $$
begin
    if new.revoked_at is null
       and exists (select 1 from tenant_members
                   where tenant_id = new.tenant_id and user_id = new.created_by
                     and left_at is not null)
    then
        raise exception 'convite aberto de membro inativo (%)', new.created_by
            using errcode = 'check_violation', constraint = 'tenant_invites_creator_active';
    end if;
    return new;
end $$;

create trigger tenant_invites_creator_active
    before insert or update on tenant_invites
    for each row execute function tenant_invites_creator_active();

-- Inscrito em torneio que não começou é membro ativo: o inativo não entra, e
-- desfazer o início tira as inscrições de quem saiu antes do commit (o status
-- volta antes, porque inscrito de torneio iniciado não sai).
create function participants_member_active() returns trigger
language plpgsql as $$
begin
    if exists (select 1 from tenant_members
               where tenant_id = new.tenant_id and user_id = new.user_id
                 and left_at is not null)
    then
        raise exception 'membro % inativo não se inscreve', new.user_id
            using errcode = 'check_violation', constraint = 'participants_member_active';
    end if;
    return new;
end $$;

create trigger participants_member_active
    before insert on participants
    for each row execute function participants_member_active();

create function tournaments_unstarted_participants() returns trigger
language plpgsql as $$
begin
    if exists (select 1 from tournaments where id = new.id)
       and not tournament_started(new.id)
       and exists (select 1 from participants p
                   join tenant_members m on m.tenant_id = p.tenant_id and m.user_id = p.user_id
                   where p.tournament_id = new.id and m.left_at is not null)
    then
        raise exception 'torneio % não começou e tem inscrito inativo', new.id
            using errcode = 'check_violation', constraint = 'participants_member_active';
    end if;
    return null;
end $$;

create constraint trigger tournaments_unstarted_participants
    after update of status on tournaments
    deferrable initially deferred
    for each row
    when (old.status in ('in_progress', 'finished') and new.status not in ('in_progress', 'finished'))
    execute function tournaments_unstarted_participants();
