-- Séries pares no round robin (05/11): melhor de 2, 4 ou 6. Fecham na maioria,
-- como as outras (2, 3 e 4 mapas) ou, com os N mapas confirmados e metade para
-- cada lado, em empate, sem vencedor: 1 x 1, 2 x 2 ou 3 x 3. Nenhum dado existente
-- muda: as regras só aceitam mais. Eliminação e suíço continuam com N ímpar, e o
-- W.O. continua com vencedor, sem placar.

-- 1. A configuração da fase: o round robin passa a aceitar 2, 4 e 6.
alter table tournament_stages
    drop constraint tournament_stages_best_of,
    add constraint tournament_stages_best_of check (
        best_of between 1 and 255
        and (best_of % 2 = 1 or (format = 'round_robin' and best_of in (2, 4, 6))));

comment on column tournament_stages.best_of is
    'Melhor de N: ímpar; no round robin, também 2, 4 ou 6. Das partidas comuns na eliminação, de todas nas fases de tabela.';

-- 2. A série decidida. O vencedor nulo é o empate da série par.
alter table match_results alter column winner drop not null;
comment on column match_results.winner is
    'Quem venceu a série; nulo só na série par empatada, com metade dos mapas para cada lado.';

alter table match_results
    drop constraint match_results_best_of_check,
    add constraint match_results_best_of_check check (
        best_of between 1 and 255
        and (best_of % 2 = 1 or (best_of in (2, 4, 6) and match_key ~ '^rr-'))),
    -- O coalesce porque, sem placar, a comparação daria nulo, e o check aceita
    -- nulo: o W.O. sem vencedor passaria.
    add constraint match_results_draw check (
        winner is not null
        or coalesce(
            best_of in (2, 4, 6) and a_wins = best_of / 2 and b_wins = best_of / 2,
            false)),
    -- Série jogada: o vencedor tem os mapas que o melhor de N pede e o perdedor
    -- menos que isso, sem passar dos mapas da série (a md2 não tem 2 x 1, nem a
    -- md4 3 x 2). O empate tem o placar em match_results_draw. W.O.: sem placar.
    drop constraint match_results_check3,
    add constraint match_results_score_check check (
        a_wins is null
        or winner is null
        or (least(a_wins, b_wins) >= 0
            and greatest(a_wins, b_wins) = best_of / 2 + 1
            and least(a_wins, b_wins) < best_of / 2 + 1
            and a_wins + b_wins <= best_of
            and (winner = side_a) = (a_wins > b_wins)));

-- 3. O histórico: a série decidida em empate fica no evento `decided` sem
-- vencedor e com o mesmo número de mapas para os dois lados.
alter table match_events
    drop constraint match_events_check1,
    add constraint match_events_maps_check check (
        winner_maps is null
        or (winner is not null and loser_maps >= 0 and winner_maps > loser_maps)
        or (kind = 'decided' and winner is null and match_key ~ '^rr-'
            and winner_maps = loser_maps and winner_maps in (1, 2, 3))),
    drop constraint match_events_outcome_check,
    add constraint match_events_outcome_check check (case
        when game is not null then
            kind in ('reported', 'confirmed', 'auto_confirmed', 'contested', 'resolved',
                     'entered', 'corrected', 'voided', 'replay')
            and winner_maps is null
            and (winner is not null) = (kind in ('reported', 'resolved', 'entered', 'corrected'))
        when kind = 'replay' then false
        -- Sem vencedor, só o empate: quem confere o placar é match_events_maps_check.
        when kind = 'decided' then winner_maps is not null
        when kind in ('reported', 'resolved', 'entered', 'corrected')
            then winner is not null and winner_maps is not null
        when kind in ('walkover', 'auto_walkover') then winner is not null and winner_maps is null
        else winner is null and winner_maps is null
    end);
