-- Disputa de posições (06/11): a fase aceita o formato placement, e as partidas
-- dele (pl-1-<n>: rodada 1, par n) entram em todo lugar que guarda o
-- identificador. Nenhum dado existente muda: só se aceita mais.
--
-- Sem mudança, conferido: tournament_stages_best_of (a disputa é sempre ímpar,
-- como o suíço), tournament_longest_series (greatest ignora o finals_best_of
-- nulo), stage_pairings_guard (pareamento só no suíço) e
-- tournament_stages_buchholz_swiss (tiebreakers nulo passa).

-- 1. O identificador de partida: wb, lb e gf da eliminação, rr do round robin,
-- sw do suíço e pl da disputa de posições. Mesmos nomes de constraint.
alter table match_results drop constraint match_results_match_key_check;
alter table match_results add constraint match_results_match_key_check
    check (match_key ~ '^(wb|lb|gf|rr|sw|pl)-[1-9][0-9]*-[1-9][0-9]*$');

alter table match_events drop constraint match_events_match_key_check;
alter table match_events add constraint match_events_match_key_check
    check (match_key ~ '^(wb|lb|gf|rr|sw|pl)-[1-9][0-9]*-[1-9][0-9]*$');

alter table match_games drop constraint match_games_match_key_check;
alter table match_games add constraint match_games_match_key_check
    check (match_key ~ '^(wb|lb|gf|rr|sw|pl)-[1-9][0-9]*-[1-9][0-9]*$');

alter table match_veto_events drop constraint match_veto_events_match_key;
alter table match_veto_events add constraint match_veto_events_match_key
    check (match_key ~ '^(wb|lb|gf|rr|sw|pl)-[1-9][0-9]*-[1-9][0-9]*$');

alter table match_screenshots drop constraint match_screenshots_match_key_check;
alter table match_screenshots add constraint match_screenshots_match_key_check
    check (match_key ~ '^(wb|lb|gf|rr|sw|pl)-[1-9][0-9]*-[1-9][0-9]*$');

alter table screenshot_deletions drop constraint screenshot_deletions_object_key_check;
alter table screenshot_deletions add constraint screenshot_deletions_object_key_check
    check (object_key ~ ('^screenshots/[a-z0-9]+(-[a-z0-9]+)*/[a-z0-9]+(-[a-z0-9]+)*/'
                         || '(wb|lb|gf|rr|sw|pl)-[1-9][0-9]*-[1-9][0-9]*/mapa-[1-9][0-9]*-'
                         || '[1-9A-HJ-NP-Za-km-z]{22}\.(webp|jpg)$'));

-- 2. O formato e as colunas dele: só best_of, como uma fase de tabela sem
-- desempate nem rodadas. O corpo do shape é o de
-- 20261211020000_stage_tiebreakers.sql, com o ramo novo.
alter table tournament_stages
    drop constraint tournament_stages_format,
    add constraint tournament_stages_format
        check (format in ('elimination', 'round_robin', 'swiss', 'placement')),
    drop constraint tournament_stages_shape,
    add constraint tournament_stages_shape check (case format
        when 'elimination' then finals_best_of is not null and bracket is not null
            and (grand_final_reset is not null) = (bracket = 'double')
            and rounds is null and tiebreakers is null
        when 'round_robin' then finals_best_of is null and bracket is null
            and grand_final_reset is null and rounds is null and tiebreakers is not null
        when 'swiss' then finals_best_of is null and bracket is null
            and grand_final_reset is null and tiebreakers is not null
        when 'placement' then finals_best_of is null and bracket is null
            and grand_final_reset is null and rounds is null and tiebreakers is null
        else false
    end);

comment on column tournament_stages.format is
    'elimination, round_robin, swiss ou placement; diz quais das outras colunas valem (tournament_stages_shape).';
comment on column tournament_stages.tiebreakers is
    'Fases de tabela: os desempates em ordem (head_to_head, map_diff, frag_diff e, só no suíço, buchholz), entre o primeiro critério e a ordem da chave, que são fixos. Vazio: decide a ordem da chave. Nulo na eliminação e na disputa de posições.';
