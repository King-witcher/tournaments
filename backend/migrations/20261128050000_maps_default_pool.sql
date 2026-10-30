-- Pool padrão de mapas do torneio e mapas que faltavam (decisões em "Torneios"
-- no docs/decisions.md).
--
-- 1. O pool padrão deixa de ser "os 7 primeiros do pool de duelo por id" e
--    passa a ser explícito: `default_position` marca os mapas do padrão e a
--    ordem deles (1 em diante). Só mapa do pool de duelo entra.
-- 2. Mapas que o jogo tem e o banco não tinha. A lista saiu dos paks da build
--    23622721 (um pak por mapa, em client/preload/paks); Crucible entra no
--    pool de duelo, depois dos que já estavam, fora do padrão. Os outros
--    ficam fora do pool de duelo. As arenas de treino do Arena Kings
--    (gridlock, ringside, warehouse, midair, divide, trench) e as variantes
--    de modo (longestyard_ctf, tower_of_koth_duel) não entram.
--
-- Torneios já criados mantêm o pool que têm (tournament_maps).

alter table maps
    add column default_position smallint unique
        constraint maps_default_position_check check (default_position between 1 and 32)
        constraint maps_default_in_duel_pool check (default_position is null or duel_pool);

insert into maps (slug, name, duel_pool) values
    ('crucible', 'Crucible', true),
    ('citadel', 'Citadel', false),
    ('cursed-soul', 'Cursed Soul', false),
    ('dark-zone', 'Dark Zone', false),
    ('fortress-of-the-deep', 'Fortress of the Deep', false),
    ('lighthouse', 'Lighthouse', false);

update maps set default_position = d.position
from (values
    ('awoken', 1),
    ('blood-covenant', 2),
    ('blood-run', 3),
    ('corrupted-keep', 4),
    ('molten-falls', 5),
    ('ruins-of-sarnath', 6),
    ('vale-of-pnath', 7)
) as d (slug, position)
where maps.slug = d.slug;
