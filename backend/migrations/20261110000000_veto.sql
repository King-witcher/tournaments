-- Veto de mapas e ban/pick de campeões ao vivo (decisões em "Veto e
-- campeões" no docs/decisions.md). O estado do veto nunca é gravado: sai do
-- replay do engine (veto::Veto) com a configuração do torneio, a melhor de N
-- da partida e o histórico de eventos (match_veto_events), na ordem do id.
--
-- Dados existentes:
-- - torneios em andamento ou encerrados ficam sem veto (modo do mapa 'off' e
--   sem fase de campeões): as partidas seguem como antes, sem "pronto";
-- - os que não começaram ganham o padrão (veto, campeões, 30 s por passo) e
--   todo torneio ganha o pool padrão: os 7 primeiros mapas ativos do pool de
--   duelo, na ordem de maps.id. Se a série mais longa de um torneio que não
--   começou passa do tamanho do pool, ele fica com o modo 'off' (o
--   organizador escolhe outro pool antes de mudar).

-- 1. Configuração por torneio.
alter table tournaments
    -- Como sai o mapa de cada jogo: veto (ban/pick da série inteira),
    -- votação a cada jogo, sorteio a cada jogo ou desligado (os jogadores
    -- combinam fora e enviam o mapa).
    add column map_mode text not null default 'veto'
        constraint tournaments_map_mode_check check (map_mode in ('veto', 'vote', 'random', 'off')),
    -- Ban/pick de campeões a cada mapa (os bans por mapa estão em
    -- champion_rules).
    add column champion_phase boolean not null default true,
    -- Segundos de cada passo do veto; no fim, a escolha é sorteada.
    add column veto_seconds integer not null default 30
        constraint tournaments_veto_seconds_check check (veto_seconds between 10 and 300);

update tournaments
set map_mode = 'off', champion_phase = false
where status in ('in_progress', 'finished');

-- 2. Pool de mapas do torneio, na ordem configurada (o veto alterna sobre
-- ela; a votação e o sorteio sorteiam dela).
create table tournament_maps (
    tournament_id bigint not null references tournaments (id) on delete cascade,
    -- Mapa global; nunca é apagado (sai do jogo com active = false).
    map_id        bigint not null references maps (id),
    position      smallint not null constraint tournament_maps_position check (position between 1 and 64),
    primary key (tournament_id, map_id),
    unique (tournament_id, position)
);

insert into tournament_maps (tournament_id, map_id, position)
select t.id, m.id, m.position
from tournaments t
cross join (
    select id, row_number() over (order by id)::smallint as position
    from maps
    where duel_pool and active
    order by id
    limit 7
) m;

-- A série mais longa de cada torneio (o maior melhor de N das fases).
create function tournament_longest_series(tournament bigint) returns integer
language sql stable as $$
    select coalesce(max(greatest((config #>> '{series,default}')::integer,
                                 (config #>> '{series,finals}')::integer)), 0)
    from tournament_stages
    where tournament_id = tournament
$$;

update tournaments t
set map_mode = 'off'
where map_mode <> 'off'
  and tournament_longest_series(t.id)
      > (select count(*) from tournament_maps p where p.tournament_id = t.id);

-- Fora do modo 'off', o pool tem pelo menos um mapa por jogo da série mais
-- longa (o mapa não se repete na série). Conferido no commit: criar e editar
-- gravam o torneio, as fases e o pool em comandos separados.
create function check_tournament_map_pool(tournament bigint) returns void
language plpgsql as $$
begin
    if exists (select 1 from tournaments where id = tournament and map_mode <> 'off')
       and tournament_longest_series(tournament)
           > (select count(*) from tournament_maps where tournament_id = tournament)
    then
        raise exception 'torneio % com menos mapas no pool que jogos na série', tournament
            using errcode = 'check_violation', constraint = 'tournament_maps_pool_size';
    end if;
end $$;

create function tournaments_map_pool() returns trigger
language plpgsql as $$
begin
    perform check_tournament_map_pool(new.id);
    return null;
end $$;

create function tournament_map_pool_changed() returns trigger
language plpgsql as $$
begin
    if tg_op <> 'INSERT' then
        perform check_tournament_map_pool(old.tournament_id);
    end if;
    if tg_op <> 'DELETE' then
        perform check_tournament_map_pool(new.tournament_id);
    end if;
    return null;
end $$;

create constraint trigger tournaments_map_pool
    after insert or update of map_mode on tournaments
    deferrable initially deferred
    for each row execute function tournaments_map_pool();

create constraint trigger tournament_maps_pool
    after insert or update or delete on tournament_maps
    deferrable initially deferred
    for each row execute function tournament_map_pool_changed();

create constraint trigger tournament_stages_map_pool
    after insert or update on tournament_stages
    deferrable initially deferred
    for each row execute function tournament_map_pool_changed();

-- O pool e a configuração do veto de torneio iniciado ficam como estão (o
-- veto das partidas sai deles). A cascata do torneio passa: sem ele,
-- tournament_started é falso.
create function tournament_maps_frozen_after_start() returns trigger
language plpgsql as $$
begin
    if tg_op <> 'INSERT' and tournament_started(old.tournament_id) then
        raise exception 'pool de mapas do torneio % já iniciado', old.tournament_id
            using errcode = 'check_violation', constraint = 'tournament_maps_started_frozen';
    end if;
    if tg_op <> 'DELETE' and tournament_started(new.tournament_id) then
        raise exception 'pool de mapas do torneio % já iniciado', new.tournament_id
            using errcode = 'check_violation', constraint = 'tournament_maps_started_frozen';
    end if;
    if tg_op = 'DELETE' then
        return old;
    end if;
    return new;
end $$;

create trigger tournament_maps_frozen_after_start
    before insert or update or delete on tournament_maps
    for each row execute function tournament_maps_frozen_after_start();

create function tournaments_veto_frozen() returns trigger
language plpgsql as $$
begin
    if old.status in ('in_progress', 'finished') and new.status in ('in_progress', 'finished')
       and (new.map_mode, new.champion_phase, new.veto_seconds)
           is distinct from (old.map_mode, old.champion_phase, old.veto_seconds)
    then
        raise exception 'torneio % já começou: configuração do veto congelada', old.id
            using errcode = 'check_violation', constraint = 'tournaments_veto_started_frozen';
    end if;
    return new;
end $$;

create trigger tournaments_veto_frozen
    before update on tournaments
    for each row execute function tournaments_veto_frozen();

-- 3. Histórico do veto de cada partida, só acrescentado (desfazer é um
-- evento). Sai inteiro quando o início do torneio é desfeito ou quando a
-- partida passa a ter outros lados (uma correção mudou quem joga).
create table match_veto_events (
    id               bigint generated always as identity primary key,
    tenant_id        bigint not null,
    tournament_id    bigint not null,
    -- Posição da fase (tournament_stages.position).
    stage            smallint not null,
    -- engine::core::MatchId legível: wb-1-3, lb-2-1, gf-1-1.
    match_key        text not null
                     constraint match_veto_events_match_key
                     check (match_key ~ '^(wb|lb|gf)-[1-9][0-9]*-[1-9][0-9]*$'),
    -- Lados da partida quando o evento entrou (iguais em todos os eventos
    -- dela).
    side_a           bigint not null,
    side_b           bigint not null,
    -- ready: um jogador está pronto (antes do veto). begin: o veto começa (os
    -- dois prontos, ou o organizador forçou) e abre o jogo 1. game: o jogo
    -- seguinte começa (o anterior foi enviado). ban, pick, vote: um passo.
    -- undo: desfaz o último passo do jogo em andamento (o organizador, ou o
    -- jogador que aceitou o pedido). undo_request e undo_declined: o pedido
    -- de desfazer de um jogador e a recusa do adversário.
    kind             text not null
                     constraint match_veto_events_kind check (kind in (
                         'ready', 'begin', 'game', 'ban', 'pick', 'vote', 'undo',
                         'undo_request', 'undo_declined')),
    -- O jogo do evento: nulo no ready; 1 no begin.
    game             smallint constraint match_veto_events_game check (game between 1 and 255),
    -- ready, ban, pick, vote, undo_request, undo_declined: o lado de quem
    -- agiu. begin: quem começa (o seed melhor). game: quem começa os
    -- campeões do jogo. undo: nulo.
    side             text constraint match_veto_events_side check (side in ('a', 'b')),
    -- Ban, pick ou voto de mapa: um mapa do pool do torneio.
    map_id           bigint,
    -- game: o mapa em que o jogo anterior foi enviado (nulo se foi sem
    -- mapa); sai da votação e do sorteio dos jogos seguintes.
    played_map_id    bigint references maps (id),
    -- Ban ou pick de campeão.
    champion_id      bigint references champions (id),
    -- begin: a seed dos sorteios da série (o u64 do engine, bit a bit).
    seed             bigint,
    -- A escolha saiu pelo fim do tempo do passo (sorteada).
    timeout          boolean not null default false,
    -- Quem agiu: o jogador, ou o organizador (begin forçado, undo). Nulo no
    -- que acontece sozinho (begin com os dois prontos, game, tempo
    -- esgotado) e se o usuário for apagado.
    actor            bigint references users (id) on delete set null,
    created_at       timestamptz not null default now(),
    foreign key (tenant_id, tournament_id) references tournaments (tenant_id, id)
        on delete cascade,
    foreign key (tournament_id, stage) references tournament_stages (tournament_id, position),
    foreign key (tournament_id, side_a) references participants (tournament_id, id),
    foreign key (tournament_id, side_b) references participants (tournament_id, id),
    foreign key (tournament_id, map_id) references tournament_maps (tournament_id, map_id),
    constraint match_veto_events_sides check (side_a <> side_b),
    -- Cada tipo com as colunas dele, e só elas.
    constraint match_veto_events_shape check (case kind
        when 'ready' then game is null and side is not null
            and map_id is null and champion_id is null
        when 'begin' then game = 1 and side is not null and seed is not null
            and map_id is null and champion_id is null
        when 'game' then game >= 2 and side is not null
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
    end),
    constraint match_veto_events_seed check ((seed is not null) = (kind = 'begin')),
    constraint match_veto_events_played_map check (played_map_id is null or kind = 'game'),
    -- Só um passo sai pelo tempo, e sem autor.
    constraint match_veto_events_timeout
        check (not timeout or (kind in ('ban', 'pick', 'vote') and actor is null))
);

create index match_veto_events_match_idx on match_veto_events (tournament_id, stage, match_key, id);

-- Um começo por partida, um pronto por lado e um início por jogo.
create unique index match_veto_events_begin_once
    on match_veto_events (tournament_id, stage, match_key) where kind = 'begin';
create unique index match_veto_events_ready_once
    on match_veto_events (tournament_id, stage, match_key, side) where kind = 'ready';
create unique index match_veto_events_game_once
    on match_veto_events (tournament_id, stage, match_key, game) where kind in ('begin', 'game');

-- O histórico só cresce, com os mesmos lados em toda a partida: pronto só
-- antes do começo, e o resto só depois dele. Apagar (a partida inteira) é
-- permitido.
create function match_veto_events_guard() returns trigger
language plpgsql as $$
declare
    begun boolean;
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
    if (new.kind = 'ready' and begun) or (new.kind not in ('ready', 'begin') and not begun) then
        raise exception 'evento % fora de ordem no veto da partida %', new.kind, new.match_key
            using errcode = 'check_violation', constraint = 'match_veto_events_order';
    end if;
    return new;
end $$;

create trigger match_veto_events_guard
    before insert or update on match_veto_events
    for each row execute function match_veto_events_guard();
