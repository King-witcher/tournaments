-- Desempate escolhido por fase (do usuário, 05/11): nas fases de tabela, a ordem
-- dos desempates que vêm depois do primeiro critério (saldo de séries no round
-- robin, pontos no suíço) e antes da ordem da chave (seed), os dois fixos. Cada
-- um no máximo uma vez; a lista vazia vale (decide a ordem da chave). Na
-- eliminação, nulo.
--
-- As fases que já existem ficam com a ordem que valia até aqui, então nenhuma
-- tabela nem pareamento muda: round robin com confronto direto, saldo de mapas
-- e saldo de frags; suíço com o Buchholz antes deles.

alter table tournament_stages add column tiebreakers text[];

-- Fase de torneio iniciado não muda (o trigger recusa), e o preenchimento passa
-- por todas: os dois triggers ficam desligados só no update, como em
-- 20261210000000_tournament_setup_columns.sql. O do pool não acrescenta nada
-- aqui e, deferido, deixaria o alter table seguinte sem rodar.
alter table tournament_stages
    disable trigger tournament_stages_frozen_after_start,
    disable trigger tournament_stages_map_pool;

update tournament_stages
set tiebreakers = case format
    when 'round_robin' then array['head_to_head', 'map_diff', 'frag_diff']
    when 'swiss' then array['buchholz', 'head_to_head', 'map_diff', 'frag_diff']
end;

alter table tournament_stages
    enable trigger tournament_stages_frozen_after_start,
    enable trigger tournament_stages_map_pool;

alter table tournament_stages
    -- Só os quatro que existem, numa lista simples (uma dimensão, começando em
    -- 1, sem nulo: o `<@` recusa o nulo), e cada um no máximo uma vez.
    add constraint tournament_stages_tiebreakers check (
        tiebreakers <@ array['head_to_head', 'map_diff', 'frag_diff', 'buchholz']
        and coalesce(array_ndims(tiebreakers), 1) = 1
        and coalesce(array_lower(tiebreakers, 1), 1) = 1
        and cardinality(array_positions(tiebreakers, 'head_to_head')) <= 1
        and cardinality(array_positions(tiebreakers, 'map_diff')) <= 1
        and cardinality(array_positions(tiebreakers, 'frag_diff')) <= 1
        and cardinality(array_positions(tiebreakers, 'buchholz')) <= 1),
    add constraint tournament_stages_buchholz_swiss
        check (format = 'swiss' or not ('buchholz' = any(tiebreakers))),
    -- O corpo é o de 20261210000000_tournament_setup_columns.sql, com a coluna
    -- nova: nula na eliminação, sempre presente nas fases de tabela.
    drop constraint tournament_stages_shape,
    add constraint tournament_stages_shape check (case format
        when 'elimination' then finals_best_of is not null and bracket is not null
            and (grand_final_reset is not null) = (bracket = 'double')
            and rounds is null and tiebreakers is null
        when 'round_robin' then finals_best_of is null and bracket is null
            and grand_final_reset is null and rounds is null and tiebreakers is not null
        when 'swiss' then finals_best_of is null and bracket is null
            and grand_final_reset is null and tiebreakers is not null
        else false
    end);

comment on column tournament_stages.tiebreakers is
    'Fases de tabela: os desempates em ordem (head_to_head, map_diff, frag_diff e, só no suíço, buchholz), entre o primeiro critério e a ordem da chave, que são fixos. Vazio: decide a ordem da chave. Nulo na eliminação.';
