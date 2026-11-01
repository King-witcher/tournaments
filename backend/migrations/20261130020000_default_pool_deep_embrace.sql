-- Pool padrão de um torneio novo (do usuário, 31/10): Awoken, Blood Covenant,
-- Corrupted Keep, Molten Falls, Ruins of Sarnath, Vale of Pnath e Deep
-- Embrace, nessa ordem. Blood Run sai do padrão e continua no pool de duelo.
-- Torneios já criados mantêm o pool que têm (tournament_maps).

-- Todos saem antes: default_position é unique.
update maps set default_position = null where default_position is not null;

update maps set default_position = d.position
from (values
    ('awoken', 1),
    ('blood-covenant', 2),
    ('corrupted-keep', 3),
    ('molten-falls', 4),
    ('ruins-of-sarnath', 5),
    ('vale-of-pnath', 6),
    ('deep-embrace', 7)
) as d (slug, position)
where maps.slug = d.slug;
