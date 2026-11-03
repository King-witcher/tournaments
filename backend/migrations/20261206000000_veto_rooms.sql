-- Sala de veto: veto de mapas e draft de campeões avulsos, sem torneio nem
-- comunidade, para quem não usa o site para gerenciar a chave. Qualquer um cria,
-- sem login; o link é a credencial (um por jogador, um de espectador e um de
-- gestão). Decisões em docs/decisions/veto.md ("Sala de veto"); os motivos de
-- ficar no Postgres e não no Redis, e dos tokens em claro, em
-- docs/decisions/stack.md.
--
-- Uma migration que apague um mapa ou um campeão precisa apagar antes as salas
-- que os usam (as FKs recusam): a sala dura no máximo 24 h.

-- A sala e a configuração dela, que não muda depois de criada: o replay lê daqui.
create table veto_rooms (
    id                bigint generated always as identity primary key,
    -- Em claro, como os dos lugares: efêmeros, e precisam ser mostrados de novo.
    manage_token      text not null unique
        constraint veto_rooms_manage_token check (manage_token ~ '^[1-9A-HJ-NP-Za-km-z]{22}$'),
    spectator_token   text not null unique
        constraint veto_rooms_spectator_token check (spectator_token ~ '^[1-9A-HJ-NP-Za-km-z]{22}$'),
    -- Quem criou; nulo: sem login, o que conta no teto de salas sem login.
    created_by        bigint references users (id) on delete set null,
    -- Jogadores por lado: 1 no duelo, de 2 a 4 nos times.
    team_size         smallint not null constraint veto_rooms_team_size
        check (team_size between 1 and 4),
    best_of           smallint not null constraint veto_rooms_best_of
        check (best_of in (1, 3, 5)),
    -- Votação e sorteio dependem do envio de cada partida, que a sala não tem.
    map_mode          text not null constraint veto_rooms_map_mode
        check (map_mode in ('veto', 'off')),
    veto_order        text not null constraint veto_rooms_veto_order
        check (veto_order in ('bans_first', 'alternating')),
    champion_phase    boolean not null,
    bans_per_series   smallint not null constraint veto_rooms_bans
        check (bans_per_series between 0 and 3),
    mirror            boolean not null,
    same_team         boolean not null,
    -- Quem não repete campeão entre as partidas da série; nulo: pode repetir.
    no_repeat         text constraint veto_rooms_no_repeat
        check (no_repeat in ('player', 'team')),
    step_seconds      integer not null constraint veto_rooms_step_seconds
        check (step_seconds between 10 and 300),
    team_pick_seconds integer not null constraint veto_rooms_team_pick_seconds
        check (team_pick_seconds between 15 and 300),
    created_at        timestamptz not null default now(),
    -- Vencida, a sala não é lida, mesmo antes de a rotina apagá-la: 2 h se
    -- ninguém começou, 24 h a partir do início.
    expires_at        timestamptz not null,
    -- A hora do próximo sorteio (o prazo do passo mais a tolerância); nulo sem
    -- passo esperando. Repete o que sai do replay para o relógio achar o próximo
    -- sem refazer todas as salas abertas; quem grava um evento grava esta coluna.
    due_at            timestamptz,
    constraint veto_rooms_has_a_phase check (map_mode <> 'off' or champion_phase)
);

create index veto_rooms_due_idx on veto_rooms (due_at) where due_at is not null;
create index veto_rooms_expires_idx on veto_rooms (expires_at);
create index veto_rooms_anonymous_idx on veto_rooms (expires_at) where created_by is null;

-- O pool de mapas da sala, na ordem. Tabela, como o do torneio, para os passos
-- apontarem por FK.
create table veto_room_maps (
    room_id  bigint not null references veto_rooms (id) on delete cascade,
    map_id   bigint not null references maps (id),
    position smallint not null constraint veto_room_maps_position
        check (position between 1 and 32),
    primary key (room_id, map_id),
    unique (room_id, position)
);

-- Um lugar por jogador, com o link dele. O id entra no veto como o jogador
-- (`PlayerId`).
create table veto_room_seats (
    id       bigint generated always as identity primary key,
    room_id  bigint not null references veto_rooms (id) on delete cascade,
    side     text not null constraint veto_room_seats_side check (side in ('a', 'b')),
    position smallint not null constraint veto_room_seats_position
        check (position between 1 and 4),
    token    text not null unique
        constraint veto_room_seats_token check (token ~ '^[1-9A-HJ-NP-Za-km-z]{22}$'),
    -- Definido uma vez, ao entrar; nulo: ninguém entrou neste lugar.
    name     text constraint veto_room_seats_name
        check (char_length(name) between 1 and 32 and name = btrim(name)),
    unique (room_id, side, position),
    unique (room_id, id)
);

-- O que muda numa sala é só a validade e o próximo prazo; num lugar, só o nome,
-- uma vez. A configuração e o pool valem no replay de cada leitura.
create function veto_rooms_frozen() returns trigger
language plpgsql as $$
begin
    if (new.id, new.manage_token, new.spectator_token, new.team_size, new.best_of, new.map_mode,
        new.veto_order, new.champion_phase, new.bans_per_series, new.mirror, new.same_team,
        new.no_repeat, new.step_seconds, new.team_pick_seconds, new.created_at)
       is distinct from
       (old.id, old.manage_token, old.spectator_token, old.team_size, old.best_of, old.map_mode,
        old.veto_order, old.champion_phase, old.bans_per_series, old.mirror, old.same_team,
        old.no_repeat, old.step_seconds, old.team_pick_seconds, old.created_at) then
        raise exception 'configuração da sala de veto % não muda', old.id
            using errcode = 'check_violation', constraint = 'veto_rooms_frozen';
    end if;
    return new;
end $$;

create trigger veto_rooms_frozen
    before update on veto_rooms
    for each row execute function veto_rooms_frozen();

create function veto_room_seats_frozen() returns trigger
language plpgsql as $$
begin
    if (new.id, new.room_id, new.side, new.position, new.token)
       is distinct from (old.id, old.room_id, old.side, old.position, old.token)
       or (old.name is not null and new.name is distinct from old.name) then
        raise exception 'lugar % da sala de veto não muda', old.id
            using errcode = 'check_violation', constraint = 'veto_room_seats_frozen';
    end if;
    return new;
end $$;

create trigger veto_room_seats_frozen
    before update on veto_room_seats
    for each row execute function veto_room_seats_frozen();

-- No commit: exatamente `team_size` lugares por lado, numerados de 1 em diante,
-- e o pool coerente com o modo do mapa (ao menos um mapa por partida no veto;
-- nenhum sem ele). A cascata de apagar a sala passa: sem a sala, não há o que
-- conferir.
create function veto_rooms_coherent() returns trigger
language plpgsql as $$
declare
    room bigint;
    size smallint;
    games smallint;
    mode text;
    maps_count bigint;
begin
    if tg_table_name = 'veto_rooms' then
        room := new.id;
    elsif tg_op = 'DELETE' then
        room := old.room_id;
    else
        room := new.room_id;
    end if;
    select team_size, best_of, map_mode into size, games, mode from veto_rooms where id = room;
    if not found then
        return null;
    end if;
    if (select count(*) from veto_room_seats
        where room_id = room and position <= size) <> 2 * size
       or (select count(*) from veto_room_seats where room_id = room) <> 2 * size then
        raise exception 'sala de veto % sem % lugares por lado', room, size
            using errcode = 'check_violation', constraint = 'veto_rooms_seats';
    end if;
    select count(*) into maps_count from veto_room_maps where room_id = room;
    if (mode = 'veto' and maps_count < games) or (mode = 'off' and maps_count > 0) then
        raise exception 'sala de veto % com o pool incoerente com o modo do mapa', room
            using errcode = 'check_violation', constraint = 'veto_rooms_pool';
    end if;
    return null;
end $$;

create constraint trigger veto_rooms_coherent
    after insert on veto_rooms
    deferrable initially deferred
    for each row execute function veto_rooms_coherent();

create constraint trigger veto_room_seats_coherent
    after insert or delete on veto_room_seats
    deferrable initially deferred
    for each row execute function veto_rooms_coherent();

create constraint trigger veto_room_maps_coherent
    after insert or delete on veto_room_maps
    deferrable initially deferred
    for each row execute function veto_rooms_coherent();

-- O histórico da sala, de onde o `engine` refaz o estado: a mesma forma de
-- `match_veto_events`, sem o que a sala não tem (partida seguinte, votação,
-- desfazer). `actor` e `player_id` são lugares da sala.
create table veto_room_events (
    id          bigint generated always as identity primary key,
    room_id     bigint not null references veto_rooms (id) on delete cascade,
    kind        text not null constraint veto_room_events_kind
        check (kind in ('ready', 'begin', 'ban', 'pick')),
    -- A partida em andamento: sempre a 1, porque o veto de mapas e o draft são
    -- da série inteira; nulo no "Pronto".
    game        smallint,
    side        text constraint veto_room_events_side check (side in ('a', 'b')),
    map_id      bigint,
    champion_id bigint references champions (id),
    -- O u64 dos sorteios, gravado bit a bit.
    seed        bigint,
    -- Sorteado pelo fim do tempo.
    timeout     boolean not null default false,
    -- O lugar que agiu; nulo no sorteio e no começo.
    actor       bigint,
    -- Nos times: o lugar da janela simultânea.
    player_id   bigint,
    -- Pick de campeão: a partida a que ele se destina.
    pick_game   smallint,
    -- Nos times: o pick veio com alguma regra cedida.
    gave_way    boolean not null default false,
    created_at  timestamptz not null default now(),
    foreign key (room_id, map_id) references veto_room_maps (room_id, map_id),
    foreign key (room_id, actor) references veto_room_seats (room_id, id),
    foreign key (room_id, player_id) references veto_room_seats (room_id, id),
    constraint veto_room_events_shape check (case kind
        when 'ready' then game is null and side is not null and actor is not null
            and map_id is null and champion_id is null
        when 'begin' then game = 1 and side is not null and seed is not null
            and map_id is null and champion_id is null
        else game = 1 and side is not null and (map_id is null) <> (champion_id is null)
    end),
    constraint veto_room_events_pick_game check (
        (pick_game is not null) = (kind = 'pick' and champion_id is not null)
        and (pick_game is null or pick_game between 1 and 5)),
    constraint veto_room_events_player check (
        player_id is null or (kind = 'pick' and champion_id is not null)),
    constraint veto_room_events_gave_way check (not gave_way or player_id is not null),
    constraint veto_room_events_timeout check (
        not timeout or (kind in ('ban', 'pick') and actor is null))
);

create index veto_room_events_room_idx on veto_room_events (room_id, id);

-- Um começo por sala e um pronto por lado.
create unique index veto_room_events_begin_once on veto_room_events (room_id) where kind = 'begin';
create unique index veto_room_events_ready_once on veto_room_events (room_id, side)
    where kind = 'ready';

-- O histórico só cresce: pronto só antes do começo, e o resto só depois dele.
-- Apagar (a sala inteira) é permitido.
create function veto_room_events_guard() returns trigger
language plpgsql as $$
declare
    begun boolean;
begin
    if tg_op = 'UPDATE' then
        raise exception 'evento % da sala de veto não muda', old.id
            using errcode = 'check_violation', constraint = 'veto_room_events_immutable';
    end if;
    begun := exists (select 1 from veto_room_events
                     where room_id = new.room_id and kind = 'begin');
    if (new.kind = 'ready' and begun) or (new.kind not in ('ready', 'begin') and not begun) then
        raise exception 'evento % fora de ordem na sala de veto %', new.kind, new.room_id
            using errcode = 'check_violation', constraint = 'veto_room_events_order';
    end if;
    return new;
end $$;

create trigger veto_room_events_guard
    before insert or update on veto_room_events
    for each row execute function veto_room_events_guard();
