-- Desempate pelo menor rating (do usuário, 06/11): nas fases de tabela, entre
-- empatados, fica à frente quem tem o rating menor. O rating é o do início do
-- torneio, congelado por time: o da comunidade muda a cada mapa e correção, e a
-- tabela precisa sair igual em todo replay.
--
-- Nenhuma fase existente muda: o critério é desligado por padrão e nenhum
-- time tem o rating congelado até o próximo início com ele ligado.

alter table teams add column rating_at_start integer;

comment on column teams.rating_at_start is
    'Rating do time congelado no início do torneio, em centésimos: a média dos jogadores (o inicial da comunidade para quem não tem), vezes 100, arredondada. Só nos torneios com o menor rating entre os desempates; nulo antes do início e nos outros. Com até 4 jogadores por time, médias diferentes nunca dão o mesmo valor.';

alter table teams
    -- Só time da chave tem o rating do início (desfazer o início o apaga).
    add constraint teams_rating_at_start_seeded
        check (rating_at_start is null or seed is not null);

-- O corpo é o de 20261119000000_teams.sql, com a coluna nova congelada junto
-- com o seed depois do início.
create or replace function teams_frozen_after_start() returns trigger
language plpgsql as $$
begin
    if tg_op = 'DELETE' then
        if tournament_started(old.tournament_id) then
            raise exception 'time % de torneio iniciado', old.id
                using errcode = 'foreign_key_violation', constraint = 'teams_started_delete';
        end if;
        return old;
    end if;
    if tg_op = 'UPDATE'
       and (new.tenant_id, new.tournament_id) is distinct from (old.tenant_id, old.tournament_id)
    then
        raise exception 'torneio do time % não muda', old.id
            using errcode = 'check_violation', constraint = 'teams_identity_immutable';
    end if;
    if tournament_started(new.tournament_id)
       and (tg_op = 'INSERT'
            or new.seed is distinct from old.seed
            or new.rating_at_start is distinct from old.rating_at_start)
    then
        raise exception 'torneio % já começou: times e seeds congelados', new.tournament_id
            using errcode = 'check_violation', constraint = 'teams_started_frozen';
    end if;
    return new;
end $$;

-- O critério novo na lista (o corpo é o de 20261211020000_stage_tiebreakers.sql,
-- com o quinto valor).
alter table tournament_stages
    drop constraint tournament_stages_tiebreakers,
    add constraint tournament_stages_tiebreakers check (
        tiebreakers <@ array['head_to_head', 'map_diff', 'frag_diff', 'buchholz', 'lower_rating']
        and coalesce(array_ndims(tiebreakers), 1) = 1
        and coalesce(array_lower(tiebreakers, 1), 1) = 1
        and cardinality(array_positions(tiebreakers, 'head_to_head')) <= 1
        and cardinality(array_positions(tiebreakers, 'map_diff')) <= 1
        and cardinality(array_positions(tiebreakers, 'frag_diff')) <= 1
        and cardinality(array_positions(tiebreakers, 'buchholz')) <= 1
        and cardinality(array_positions(tiebreakers, 'lower_rating')) <= 1);

comment on column tournament_stages.tiebreakers is
    'Fases de tabela: os desempates em ordem (head_to_head, map_diff, frag_diff, lower_rating e, só no suíço, buchholz), entre o primeiro critério e a ordem da chave, que são fixos. Vazio: decide a ordem da chave. Nulo na eliminação.';
