-- O resultado que a contestação propõe, opcional: o mapa e os frags de cada lado.
-- Nulos sem proposta; só existem com a partida contestada e saem ao resolver. O
-- resultado proposto não entra em match_events: o evento `resolved` registra o que
-- passou a valer. As linhas existentes ficam sem proposta.
alter table match_games
    add column contest_map_id  bigint references maps (id),
    add column contest_a_frags smallint check (contest_a_frags between -999 and 999),
    add column contest_b_frags smallint check (contest_b_frags between -999 and 999),
    add constraint match_games_contest_proposal check (
        (contest_map_id is null) = (contest_a_frags is null)
        and (contest_a_frags is null) = (contest_b_frags is null)
    ),
    add constraint match_games_contest_proposal_status
        check (contest_map_id is null or status = 'contested'),
    -- Como em todo resultado, quem fez mais venceu: não há empate.
    add constraint match_games_contest_proposal_tied
        check (contest_a_frags is null or contest_a_frags <> contest_b_frags);
