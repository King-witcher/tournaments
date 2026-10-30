-- Frags negativos: no Quake Champions, suicídio e queda tiram frag, então o
-- placar de um lado num mapa pode ficar abaixo de zero (20 a -1 é comum num
-- duelo desigual). A faixa passa de 0..999 a -999..999, simétrica ao teto (a
-- mesma de MAX_FRAGS e MIN_FRAGS no matches/outcome.rs).
--
-- Só alarga a faixa: as linhas que existem já cabem nela. As demais regras dos
-- frags (os dois ou nenhum, só com o mapa, sem empate e com mais frags para
-- quem venceu, em match_games_frags_winner) valem igual com negativos.

alter table match_games
    drop constraint match_games_a_frags_check,
    drop constraint match_games_b_frags_check,
    add constraint match_games_a_frags_check check (a_frags between -999 and 999),
    add constraint match_games_b_frags_check check (b_frags between -999 and 999);
