-- Séries pares sem lista fixa (do usuário, 06/11: "md par até o número que fizer
-- sentido"): o round robin aceita qualquer N par, com o mesmo teto do ímpar,
-- em vez de só 2, 4 e 6 (20261210010000_even_series.sql). Eliminação, suíço e
-- disputa de posições continuam só com N ímpar. Só afrouxa regras: nenhuma linha
-- gravada fica inválida.

alter table tournament_stages
    drop constraint tournament_stages_best_of,
    add constraint tournament_stages_best_of check (
        best_of between 1 and 255
        and (best_of % 2 = 1 or format = 'round_robin'));

comment on column tournament_stages.best_of is
    'Melhor de N: ímpar; no round robin, também par. Das partidas comuns na eliminação, de todas nas outras fases.';

alter table match_results
    drop constraint match_results_best_of_check,
    add constraint match_results_best_of_check check (
        best_of between 1 and 255
        and (best_of % 2 = 1 or match_key ~ '^rr-')),
    -- O coalesce porque, sem placar, a comparação daria nulo, e o check aceita
    -- nulo: o W.O. sem vencedor passaria.
    drop constraint match_results_draw,
    add constraint match_results_draw check (
        winner is not null
        or coalesce(
            best_of % 2 = 0 and a_wins = best_of / 2 and b_wins = best_of / 2,
            false));

alter table match_events
    drop constraint match_events_maps_check,
    add constraint match_events_maps_check check (
        winner_maps is null
        or (winner is not null and loser_maps >= 0 and winner_maps > loser_maps)
        or (kind = 'decided' and winner is null and match_key ~ '^rr-'
            and winner_maps = loser_maps and winner_maps >= 1));
