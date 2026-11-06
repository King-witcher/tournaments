-- Desfazer o "Pronto" antes do começo, na série de um torneio e na sala de veto.
-- O histórico continua só crescendo: desfazer é um evento (unready), e está
-- pronto quem tem como último pronto ou desfazer, no lado, o pronto. Sai o
-- índice de um pronto por jogador (depois de desfazer, ele marca de novo); no
-- lugar, o guard alterna os dois: pronto só de quem não está pronto, desfazer só
-- de quem está. Concorrência: quem grava trava o torneio ou a sala antes. Nenhum
-- dado muda: só há prontos, um por jogador, e o novo tipo ainda não existe.

-- 1. Série de torneio. A forma é a de 20261129030000_series_draft.sql, com o
-- unready igual ao ready.
alter table match_veto_events
    drop constraint match_veto_events_kind,
    add constraint match_veto_events_kind check (kind in (
        'ready', 'unready', 'begin', 'game', 'ban', 'pick', 'vote', 'undo',
        'undo_request', 'undo_declined')),
    drop constraint match_veto_events_shape,
    add constraint match_veto_events_shape check (case kind
        when 'ready' then game is null and side is not null
            and map_id is null and champion_id is null
        when 'unready' then game is null and side is not null
            and map_id is null and champion_id is null
        when 'begin' then game = 1 and side is not null and seed is not null
            and map_id is null and champion_id is null
        when 'game' then game >= 2 and side is null
            and map_id is null and champion_id is null
        when 'vote' then game is not null and side is not null
            and map_id is not null and champion_id is null
        when 'ban' then game is not null and side is not null
            and (map_id is null) <> (champion_id is null)
        when 'pick' then game is not null and side is not null
            and (map_id is null) <> (champion_id is null)
        when 'undo' then game is not null and side is null
            and map_id is null and champion_id is null
        else game is not null and side is not null
            and map_id is null and champion_id is null
    end);

drop index match_veto_events_ready_once;

comment on column match_veto_events.kind is
    'ready/unready: um jogador marca ou desfaz o "Pronto" (antes do veto, alternando). begin: o veto começa. game: a partida seguinte começa. ban, pick, vote: um passo. undo: desfaz o último passo. undo_request e undo_declined: o pedido de desfazer e a recusa.';

-- O corpo é o de 20261207010000_pick_order.sql, com o unready na ordem e a
-- alternância. Sem autor (conta excluída), o pronto não conta e não se desfaz.
create or replace function match_veto_events_guard() returns trigger
language plpgsql as $$
declare
    begun boolean;
    marked boolean;
begin
    if tg_op = 'UPDATE' then
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

-- 2. A sala: o mesmo, com o lugar no papel do jogador. A forma é a de
-- 20261206000000_veto_rooms.sql, com o unready igual ao ready.
alter table veto_room_events
    drop constraint veto_room_events_kind,
    add constraint veto_room_events_kind
        check (kind in ('ready', 'unready', 'begin', 'ban', 'pick')),
    drop constraint veto_room_events_shape,
    add constraint veto_room_events_shape check (case kind
        when 'ready' then game is null and side is not null and actor is not null
            and map_id is null and champion_id is null
        when 'unready' then game is null and side is not null and actor is not null
            and map_id is null and champion_id is null
        when 'begin' then game = 1 and side is not null and seed is not null
            and map_id is null and champion_id is null
        else game = 1 and side is not null and (map_id is null) <> (champion_id is null)
    end);

drop index veto_room_events_ready_once;

-- O corpo é o de 20261206000000_veto_rooms.sql, com o unready na ordem e a
-- alternância.
create or replace function veto_room_events_guard() returns trigger
language plpgsql as $$
declare
    begun boolean;
    marked boolean;
begin
    if tg_op = 'UPDATE' then
        raise exception 'evento % da sala de veto não muda', old.id
            using errcode = 'check_violation', constraint = 'veto_room_events_immutable';
    end if;
    begun := exists (select 1 from veto_room_events
                     where room_id = new.room_id and kind = 'begin');
    if (new.kind in ('ready', 'unready') and begun)
       or (new.kind not in ('ready', 'unready', 'begin') and not begun) then
        raise exception 'evento % fora de ordem na sala de veto %', new.kind, new.room_id
            using errcode = 'check_violation', constraint = 'veto_room_events_order';
    end if;
    if new.kind in ('ready', 'unready') then
        marked := coalesce((select e.kind = 'ready' from veto_room_events e
                            where e.room_id = new.room_id and e.actor = new.actor
                              and e.kind in ('ready', 'unready')
                            order by e.id desc
                            limit 1), false);
        if marked = (new.kind = 'ready') then
            raise exception '% repetido do lugar % na sala de veto %', new.kind, new.actor,
                    new.room_id
                using errcode = 'check_violation', constraint = 'veto_room_events_ready_toggle';
        end if;
    end if;
    return new;
end $$;
