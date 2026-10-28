-- Desistência num torneio em andamento e W.O. automático (decisões em
-- "Resultados" no docs/decisions.md). O inscrito que desiste (pelo
-- organizador, ou porque saiu da comunidade) continua na chave; cada partida
-- pronta com ele, agora ou depois, vira W.O. para o adversário, gravado como
-- automático (sem autor).
--
-- Colunas novas e nulas, e checks que só alargam os de antes: nenhum dado
-- existente viola as regras abaixo. Os nomes dos checks trocados
-- (match_results_check6, match_events_check2) são os que o Postgres gerou na
-- migration de resultados.

-- 1. Desistência.
alter table participants
    -- Quando desistiu (nulo: segue no torneio). Só com o torneio em andamento,
    -- só quem está na chave, e não se desfaz.
    add column withdrawn_at timestamptz,
    -- Organizador que marcou (nulo: saiu da comunidade sozinho). Sai com o autor.
    add column withdrawn_by bigint,
    add constraint participants_withdrawn_by_fkey foreign key (tenant_id, withdrawn_by)
        references tenant_members (tenant_id, user_id) on delete set null (withdrawn_by),
    add constraint participants_withdrawn_by_check
        check (withdrawn_by is null or withdrawn_at is not null),
    add constraint participants_withdrawn_seeded check (withdrawn_at is null or seed is not null);

create index participants_withdrawn_by_idx on participants (tenant_id, withdrawn_by)
    where withdrawn_by is not null;

create function participants_withdrawal() returns trigger
language plpgsql as $$
begin
    if tg_op = 'UPDATE' and old.withdrawn_at is not null
       and new.withdrawn_at is distinct from old.withdrawn_at
    then
        raise exception 'a desistência do inscrito % não se desfaz', old.id
            using errcode = 'check_violation', constraint = 'participants_withdrawal_final';
    end if;
    if new.withdrawn_at is not null
       and (tg_op = 'INSERT' or old.withdrawn_at is null)
       and not exists (select 1 from tournaments
                       where id = new.tournament_id and status = 'in_progress')
    then
        raise exception 'desistência só com o torneio % em andamento', new.tournament_id
            using errcode = 'check_violation', constraint = 'participants_withdrawal_in_progress';
    end if;
    return new;
end $$;

create trigger participants_withdrawal
    before insert or update of withdrawn_at on participants
    for each row execute function participants_withdrawal();

-- Torneio que volta a não ter começado (desfazer o início): os desistentes
-- saem da inscrição antes do commit, como os inscritos que saíram da
-- comunidade.
create or replace function tournaments_unstarted_participants() returns trigger
language plpgsql as $$
begin
    if exists (select 1 from tournaments where id = new.id)
       and not tournament_started(new.id)
    then
        if exists (select 1 from participants p
                   join tenant_members m on m.tenant_id = p.tenant_id and m.user_id = p.user_id
                   where p.tournament_id = new.id and m.left_at is not null)
        then
            raise exception 'torneio % não começou e tem inscrito inativo', new.id
                using errcode = 'check_violation', constraint = 'participants_member_active';
        end if;
        if exists (select 1 from participants
                   where tournament_id = new.id and withdrawn_at is not null)
        then
            raise exception 'torneio % não começou e tem desistente', new.id
                using errcode = 'check_violation', constraint = 'participants_withdrawn_started';
        end if;
    end if;
    return null;
end $$;

-- 2. W.O. automático: confirmado "pela desistência", sem placar, sem quem
-- enviou e sem quem confirmou; o perdedor desistiu ou saiu da comunidade.
alter table match_results
    drop constraint match_results_confirmed_via_check,
    add constraint match_results_confirmed_via_check
        check (confirmed_via in ('opponent', 'deadline', 'organizer', 'withdrawal')),
    drop constraint match_results_check6,
    add constraint match_results_reported_check
        check (reported_side is not null or confirmed_via in ('organizer', 'withdrawal')),
    add constraint match_results_withdrawal_check
        check (confirmed_via <> 'withdrawal'
               or (a_wins is null and reported_side is null
                   and reported_by is null and confirmed_by is null));

create function match_results_withdrawal_loser() returns trigger
language plpgsql as $$
begin
    if not exists (
        select 1 from participants p
        join tenant_members m on m.tenant_id = p.tenant_id and m.user_id = p.user_id
        where p.tournament_id = new.tournament_id
          and p.id = case when new.winner = new.side_a then new.side_b else new.side_a end
          and (p.withdrawn_at is not null or m.left_at is not null))
    then
        raise exception 'W.O. automático em % sem desistente', new.match_key
            using errcode = 'check_violation', constraint = 'match_results_withdrawal_loser';
    end if;
    return new;
end $$;

create trigger match_results_withdrawal_loser
    before insert or update on match_results
    for each row when (new.confirmed_via = 'withdrawal')
    execute function match_results_withdrawal_loser();

-- O evento do W.O. automático: só o vencedor, sem autor.
alter table match_events
    drop constraint match_events_kind_check,
    add constraint match_events_kind_check check (kind in (
        'reported', 'confirmed', 'auto_confirmed', 'contested', 'resolved',
        'entered', 'corrected', 'walkover', 'auto_walkover', 'voided')),
    drop constraint match_events_check2,
    add constraint match_events_outcome_check check (case
        when kind in ('reported', 'resolved', 'entered', 'corrected')
            then winner is not null and winner_maps is not null
        when kind in ('walkover', 'auto_walkover') then winner is not null and winner_maps is null
        else winner is null and winner_maps is null
    end),
    add constraint match_events_auto_walkover_actor check (kind <> 'auto_walkover' or actor is null);
