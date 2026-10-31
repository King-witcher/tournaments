-- Draft de campeões da série inteira antes da partida 1, com bans da série,
-- espelho, repetição no time e não repetição entre partidas; qualquer jogador
-- do time age pelo time (decisões em "Veto e campeões" e "TDM" no
-- docs/decisions.md).
--
-- Decisão do usuário (30/10): a produção ainda não tem usuários, então os
-- torneios são apagados em vez de convertidos (a regra de campeões e o veto
-- gravados mudaram de forma). O staging perde os de teste.

-- 1. Apagar os torneios, como em 20261119000000_teams.sql: a cascata leva
-- tudo do torneio, e o rating é refeito do zero.
delete from tournaments;
set constraints all immediate;
delete from rating_history;
delete from ratings;
insert into rating_dirty (tenant_id) select id from tenants on conflict do nothing;

-- 2. A regra de campeões: bans de cada lado na série (0 a 3), espelho
-- (adversários com o mesmo campeão na partida), repetição no time (colegas
-- com o mesmo campeão na partida) e não repetição entre partidas, com o
-- escopo "series" no lugar de "match".
create or replace function champion_rules_valid(rules jsonb) returns boolean
language sql immutable as $$
    select coalesce(
        jsonb_typeof(rules) = 'object'
        and jsonb_typeof(rules -> 'bans_per_series') = 'number'
        and rules ->> 'bans_per_series' in ('0', '1', '2', '3')
        and jsonb_typeof(rules -> 'mirror') = 'boolean'
        and jsonb_typeof(rules -> 'same_team') = 'boolean'
        and case jsonb_typeof(rules -> 'no_repeat')
            when 'null' then true
            when 'object' then rules #>> '{no_repeat,who}' in ('player', 'team')
                and rules #>> '{no_repeat,scope}' in ('series', 'stage', 'tournament')
            else false
        end,
        false)
$$;

-- 3. O veto: o draft da série é feito com a partida 1 aberta (game é a
-- partida em andamento), então o pick de campeão diz a partida a que se
-- destina (pick_game, a em andamento ou uma seguinte), e o da janela
-- simultânea do TDM diz se veio com alguma regra cedida (gave_way: dois picks
-- iguais na partida não são erro). O "game" (a partida seguinte começa) não
-- diz mais quem começa os campeões: side nulo.
alter table match_veto_events
    add column pick_game smallint,
    add column gave_way boolean not null default false,
    drop constraint match_veto_events_shape,
    add constraint match_veto_events_shape check (case kind
        when 'ready' then game is null and side is not null
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
    end),
    add constraint match_veto_events_pick_game check (
        (pick_game is not null) = (kind = 'pick' and champion_id is not null)
        and (pick_game is null or pick_game between game and 255)),
    add constraint match_veto_events_gave_way check (not gave_way or player_id is not null);
