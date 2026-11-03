-- O nome do mapa estava errado: é Vestibule of Exile, não Exile. Os mapas são
-- referenciados por id, então só o slug e o nome mudam.
update maps
set slug = 'vestibule-of-exile', name = 'Vestibule of Exile'
where slug = 'exile';
