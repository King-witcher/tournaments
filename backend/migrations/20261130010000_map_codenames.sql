-- Fortress of the Deep e Lighthouse (migration 20261128050000) não são mapas:
-- são os nomes internos dos paks de Molten Falls e Tempest Shrine, os dois
-- mapas que pareciam sem pak na build 23622721 (o pak lighthouse traz o
-- slipgate de tempest_shrine). Não saíram do jogo, então não ficam com
-- active = false: o que aponta para eles passa para o mapa de verdade e as
-- duas linhas saem.

create temporary table map_codenames on commit drop as
select codename.id as old_id, real.id as new_id
from (values
    ('fortress-of-the-deep', 'molten-falls'),
    ('lighthouse', 'tempest-shrine')
) as c (old_slug, new_slug)
join maps codename on codename.slug = c.old_slug
join maps real on real.slug = c.new_slug;

-- 1. O torneio que tinha o codinome sem o mapa de verdade ganha o mapa, no
-- fim do pool por enquanto (o veto aponta para o pool do torneio).
create temporary table map_codenames_added on commit drop as
select tm.tournament_id, c.new_id, tm.position as old_position,
       ((select max(p.position) from tournament_maps p where p.tournament_id = tm.tournament_id)
        + row_number() over (partition by tm.tournament_id order by tm.position))::smallint
           as temp_position
from tournament_maps tm
join map_codenames c on c.old_id = tm.map_id
where not exists (
    select 1 from tournament_maps r
    where r.tournament_id = tm.tournament_id and r.map_id = c.new_id
);

insert into tournament_maps (tournament_id, map_id, position)
select tournament_id, new_id, temp_position from map_codenames_added;

-- 2. O veto e os jogos passam para o mapa de verdade.
update match_veto_events e set map_id = c.new_id
from map_codenames c where e.map_id = c.old_id;

update match_veto_events e set played_map_id = c.new_id
from map_codenames c where e.played_map_id = c.old_id;

update match_games g set map_id = c.new_id
from map_codenames c where g.map_id = c.old_id;

-- 3. O codinome sai do pool, e o mapa acrescentado no passo 1 fica no lugar
-- dele. Quem tinha os dois fica com o mapa de verdade onde estava.
delete from tournament_maps tm
using map_codenames c
where tm.map_id = c.old_id;

update tournament_maps tm set position = a.old_position
from map_codenames_added a
where tm.tournament_id = a.tournament_id and tm.map_id = a.new_id;

delete from maps m
using map_codenames c
where m.id = c.old_id;
