-- Quem age pelo time (decisões em docs/decisions/veto.md, "Quem age pelo time").
-- Quatro regras no torneio em time: vetar mapas (com o voto), banir campeões,
-- escolher campeões e enviar o resultado (com confirmar e contestar). Só as
-- regras ficam gravadas, nunca o preset: o front o deriva delas, e um preset novo
-- não pede migration. O "Pronto" passa a contar por jogador. Na sala de veto, um
-- valor só, e a sala centrada no capitão tem um link por time.

-- 1. As regras. O default é o comportamento de antes (Todos os jogadores), para
-- quem já existe; o formulário de um torneio novo em time manda o capitão.
alter table tournaments
    add column veto_by text not null default 'anyone'
        constraint tournaments_veto_by check (veto_by in ('captain', 'anyone')),
    add column ban_by text not null default 'anyone'
        constraint tournaments_ban_by check (ban_by in ('captain', 'anyone')),
    add column pick_by text not null default 'each_player'
        constraint tournaments_pick_by check (pick_by in ('captain', 'each_player')),
    add column report_by text not null default 'anyone'
        constraint tournaments_report_by check (report_by in ('captain', 'anyone'));

comment on column tournaments.veto_by is
    'Quem veta os mapas e vota pelo time: captain ou anyone. Sem efeito no duelo.';
comment on column tournaments.ban_by is
    'Quem bane campeões pelo time: captain ou anyone. Sem efeito no duelo.';
comment on column tournaments.pick_by is
    'Quem escolhe os campeões: captain, por todos do time, ou each_player, cada um o próprio. Sem efeito no duelo.';
comment on column tournaments.report_by is
    'Quem envia, confirma e contesta o resultado pelo time: captain ou anyone. Sem efeito no duelo.';

-- 2. As regras congelam com o resto da configuração depois do início, a de
-- resultado inclusive (o corpo é o de 20261207010000_pick_order.sql, mais as
-- colunas novas). As séries já começadas leem a linha atual.
create or replace function tournaments_veto_frozen() returns trigger
language plpgsql as $$
begin
    if old.status in ('in_progress', 'finished') and new.status in ('in_progress', 'finished')
       and (new.map_mode, new.champion_phase, new.veto_seconds, new.team_pick_seconds,
            new.veto_order, new.pick_order, new.veto_by, new.ban_by, new.pick_by, new.report_by)
           is distinct from (old.map_mode, old.champion_phase, old.veto_seconds,
                             old.team_pick_seconds, old.veto_order, old.pick_order,
                             old.veto_by, old.ban_by, old.pick_by, old.report_by)
    then
        raise exception 'torneio % já começou: configuração do veto congelada', old.id
            using errcode = 'check_violation', constraint = 'tournaments_veto_started_frozen';
    end if;
    if old.status in ('in_progress', 'finished') and new.status in ('in_progress', 'finished')
       and (new.mode, new.team_size) is distinct from (old.mode, old.team_size)
    then
        raise exception 'torneio % já começou: modo e tamanho do time congelados', old.id
            using errcode = 'check_violation', constraint = 'tournaments_mode_started_frozen';
    end if;
    return new;
end $$;

-- 3. O "Pronto" é por jogador: um por pessoa em cada lado, não mais um por lado.
-- Quem já tinha marcado continua marcado, e a API conta sobre o elenco de agora.
drop index match_veto_events_ready_once;
create unique index match_veto_events_ready_once
    on match_veto_events (tournament_id, stage, match_key, side, actor) where kind = 'ready';

-- 4. A sala. captain: um link por time, o do lugar 1, que age por todos os lugares
-- do lado. all: um link por lugar, como antes, que é o que fica nas salas que já
-- existem. O duelo fica em all: um lugar por lado.
alter table veto_rooms
    add column team_actors text not null default 'all'
        constraint veto_rooms_team_actors check (team_actors in ('captain', 'all'));

comment on column veto_rooms.team_actors is
    'Quem age pelo time: captain = só o lugar 1 de cada lado tem link e faz tudo; all = um link por lugar.';

-- O lugar sem link: o token nulo. Ninguém entra por ele, e o `= $1` das consultas
-- por token nunca o acha.
alter table veto_room_seats alter column token drop not null;

drop index veto_room_events_ready_once;
create unique index veto_room_events_ready_once on veto_room_events (room_id, actor)
    where kind = 'ready';

-- O corpo é o de 20261207010000_pick_order.sql, mais a coluna nova.
create or replace function veto_rooms_frozen() returns trigger
language plpgsql as $$
begin
    if (new.id, new.manage_token, new.spectator_token, new.team_size, new.best_of, new.map_mode,
        new.veto_order, new.champion_phase, new.bans_per_series, new.mirror, new.same_team,
        new.no_repeat, new.step_seconds, new.team_pick_seconds, new.created_at, new.pick_order,
        new.who_starts, new.team_actors)
       is distinct from
       (old.id, old.manage_token, old.spectator_token, old.team_size, old.best_of, old.map_mode,
        old.veto_order, old.champion_phase, old.bans_per_series, old.mirror, old.same_team,
        old.no_repeat, old.step_seconds, old.team_pick_seconds, old.created_at, old.pick_order,
        old.who_starts, old.team_actors) then
        raise exception 'configuração da sala de veto % não muda', old.id
            using errcode = 'check_violation', constraint = 'veto_rooms_frozen';
    end if;
    return new;
end $$;

-- No commit, como os lugares e o pool: só falta link no lugar de sala centrada no
-- capitão que não é o primeiro do lado (o corpo é o de 20261206000000_veto_rooms.sql,
-- mais essa conferência).
create or replace function veto_rooms_coherent() returns trigger
language plpgsql as $$
declare
    room bigint;
    size smallint;
    games smallint;
    mode text;
    actors text;
    maps_count bigint;
begin
    if tg_table_name = 'veto_rooms' then
        room := new.id;
    elsif tg_op = 'DELETE' then
        room := old.room_id;
    else
        room := new.room_id;
    end if;
    select team_size, best_of, map_mode, team_actors into size, games, mode, actors
    from veto_rooms where id = room;
    if not found then
        return null;
    end if;
    if (select count(*) from veto_room_seats
        where room_id = room and position <= size) <> 2 * size
       or (select count(*) from veto_room_seats where room_id = room) <> 2 * size then
        raise exception 'sala de veto % sem % lugares por lado', room, size
            using errcode = 'check_violation', constraint = 'veto_rooms_seats';
    end if;
    if exists (select 1 from veto_room_seats
               where room_id = room
                 and (token is null) <> (actors = 'captain' and position > 1)) then
        raise exception 'sala de veto % com os links incoerentes com quem age', room
            using errcode = 'check_violation', constraint = 'veto_rooms_seat_links';
    end if;
    select count(*) into maps_count from veto_room_maps where room_id = room;
    if (mode = 'veto' and maps_count < games) or (mode = 'off' and maps_count > 0) then
        raise exception 'sala de veto % com o pool incoerente com o modo do mapa', room
            using errcode = 'check_violation', constraint = 'veto_rooms_pool';
    end if;
    return null;
end $$;
