-- As guardas de "não muda" deixam passar o `set null` da autoria: apagar o autor
-- zera a coluna por um UPDATE da própria FK, e o trigger, que recusava todo
-- UPDATE, fazia a exclusão falhar. É o caso de `decided_by` (apagar o membro, e
-- a comunidade em cascata, que o agendador de exclusão tentaria a cada passada)
-- e de `actor` (apagar o usuário). Comparar em jsonb, sem a coluna, vale também
-- para as colunas que vierem depois. Nenhuma linha muda.

-- 1. A decisão de empate: o corpo é o de 20261215010000_stage_final_tie.sql.
create or replace function stage_tie_decisions_guard() returns trigger
language plpgsql as $$
begin
    if tg_op = 'UPDATE' then
        if old.decided_by is not null and new.decided_by is null
           and to_jsonb(new) - 'decided_by' = to_jsonb(old) - 'decided_by' then
            return new;
        end if;
        raise exception 'decisão de empate % não muda', old.id
            using errcode = 'check_violation', constraint = 'stage_tie_decisions_immutable';
    end if;
    if not exists (select 1 from tournament_stages
                   where tournament_id = new.tournament_id and position = new.stage
                     and final_tie = 'organizer') then
        raise exception 'decisão de empate numa fase sem a organização decidindo (%)',
                new.tournament_id
            using errcode = 'check_violation', constraint = 'stage_tie_decisions_organizer';
    end if;
    if not tournament_started(new.tournament_id) then
        raise exception 'decisão de empate em torneio não iniciado (%)', new.tournament_id
            using errcode = 'check_violation', constraint = 'stage_tie_decisions_started';
    end if;
    return new;
end $$;

-- 2. O histórico do veto: o corpo é o de 20261213000000_veto_unready.sql.
create or replace function match_veto_events_guard() returns trigger
language plpgsql as $$
declare
    begun boolean;
    marked boolean;
begin
    if tg_op = 'UPDATE' then
        if old.actor is not null and new.actor is null
           and to_jsonb(new) - 'actor' = to_jsonb(old) - 'actor' then
            return new;
        end if;
        raise exception 'evento de veto % não muda', old.id
            using errcode = 'check_violation', constraint = 'match_veto_events_immutable';
    end if;
    if exists (select 1 from match_veto_events
               where tournament_id = new.tournament_id and stage = new.stage
                 and match_key = new.match_key
                 and (side_a, side_b) <> (new.side_a, new.side_b)) then
        raise exception 'veto da partida % com outros lados', new.match_key
            using errcode = 'check_violation', constraint = 'match_veto_events_same_sides';
    end if;
    begun := exists (select 1 from match_veto_events
                     where tournament_id = new.tournament_id and stage = new.stage
                       and match_key = new.match_key and kind = 'begin');
    if (new.kind in ('ready', 'unready') and begun)
       or (new.kind not in ('ready', 'unready', 'begin') and not begun) then
        raise exception 'evento % fora de ordem no veto da partida %', new.kind, new.match_key
            using errcode = 'check_violation', constraint = 'match_veto_events_order';
    end if;
    if new.kind in ('ready', 'unready') then
        marked := coalesce((select e.kind = 'ready' from match_veto_events e
                            where e.tournament_id = new.tournament_id and e.stage = new.stage
                              and e.match_key = new.match_key and e.side = new.side
                              and e.actor = new.actor and e.kind in ('ready', 'unready')
                            order by e.id desc
                            limit 1), false);
        if marked = (new.kind = 'ready') then
            raise exception '% repetido de % no veto da partida %', new.kind, new.actor,
                    new.match_key
                using errcode = 'check_violation', constraint = 'match_veto_events_ready_toggle';
        end if;
    end if;
    if new.player_id is not null
       and not exists (select 1 from participants p
                       where p.tournament_id = new.tournament_id and p.user_id = new.player_id
                         and p.team_id = case new.side when 'a' then new.side_a
                                                       else new.side_b end)
    then
        raise exception 'jogador % fora do time do lado % no veto de %', new.player_id,
                new.side, new.match_key
            using errcode = 'check_violation', constraint = 'match_veto_events_player_team';
    end if;
    return new;
end $$;
