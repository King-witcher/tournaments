-- Round robin e suíço (29/10): a fase aceita os dois formatos novos, as
-- partidas deles (rr-<rodada>-<n> e sw-<rodada>-<n>) entram em todo lugar
-- que guarda o identificador, e o suíço grava o pareamento de cada rodada
-- (o replay usa os gravados: corrigir um resultado antigo nunca refaz uma
-- rodada já pareada). Nenhum dado existente muda: só se aceita mais.

-- 1. A configuração da fase, com a forma do engine (formats::StageFormat):
-- {"format": "elimination", ...} como antes,
-- {"format": "round_robin", "best_of": N} e
-- {"format": "swiss", "best_of": N, "rounds": null | 1..127} (null: ⌈log2 N⌉).
create or replace function stage_config_valid(config jsonb) returns boolean
language sql immutable as $$
    select coalesce(
        jsonb_typeof(config) = 'object'
        and case config ->> 'format'
            when 'elimination' then
                jsonb_typeof(config -> 'kind') = 'object'
                and case config #>> '{kind,kind}'
                    when 'single' then true
                    when 'double' then
                        jsonb_typeof(config #> '{kind,grand_final_reset}') = 'boolean'
                    else false
                end
                and jsonb_typeof(config -> 'series') = 'object'
                and best_of_valid(config #> '{series,default}')
                and best_of_valid(config #> '{series,finals}')
            when 'round_robin' then best_of_valid(config -> 'best_of')
            when 'swiss' then
                best_of_valid(config -> 'best_of')
                and case
                    when jsonb_typeof(config -> 'rounds') = 'null' then true
                    when jsonb_typeof(config -> 'rounds') = 'number'
                         and config ->> 'rounds' ~ '^[1-9][0-9]{0,2}$'
                        then (config ->> 'rounds')::integer <= 127
                    else false
                end
            else false
        end,
        false)
$$;

-- 2. O identificador de partida: wb, lb e gf da eliminação, rr do round
-- robin e sw do suíço. Mesmos nomes de constraint.
alter table match_results drop constraint match_results_match_key_check;
alter table match_results add constraint match_results_match_key_check
    check (match_key ~ '^(wb|lb|gf|rr|sw)-[1-9][0-9]*-[1-9][0-9]*$');

alter table match_events drop constraint match_events_match_key_check;
alter table match_events add constraint match_events_match_key_check
    check (match_key ~ '^(wb|lb|gf|rr|sw)-[1-9][0-9]*-[1-9][0-9]*$');

alter table match_games drop constraint match_games_match_key_check;
alter table match_games add constraint match_games_match_key_check
    check (match_key ~ '^(wb|lb|gf|rr|sw)-[1-9][0-9]*-[1-9][0-9]*$');

alter table match_veto_events drop constraint match_veto_events_match_key;
alter table match_veto_events add constraint match_veto_events_match_key
    check (match_key ~ '^(wb|lb|gf|rr|sw)-[1-9][0-9]*-[1-9][0-9]*$');

alter table match_screenshots drop constraint match_screenshots_match_key_check;
alter table match_screenshots add constraint match_screenshots_match_key_check
    check (match_key ~ '^(wb|lb|gf|rr|sw)-[1-9][0-9]*-[1-9][0-9]*$');

alter table screenshot_deletions drop constraint screenshot_deletions_object_key_check;
alter table screenshot_deletions add constraint screenshot_deletions_object_key_check
    check (object_key ~ ('^screenshots/[a-z0-9]+(-[a-z0-9]+)*/[a-z0-9]+(-[a-z0-9]+)*/'
                         || '(wb|lb|gf|rr|sw)-[1-9][0-9]*-[1-9][0-9]*/mapa-[1-9][0-9]*-'
                         || '[1-9A-HJ-NP-Za-km-z]{22}\.(webp|jpg)$'));

-- 3. Os pareamentos do suíço, rodada a rodada. O engine gera o da rodada
-- seguinte quando todas as séries das já pareadas estão decididas, e a API
-- grava aqui; o replay da fase usa os gravados. Só inserido (nunca muda) e
-- apagado só ao desfazer o início (ou com o torneio).
create table stage_pairings (
    tenant_id     bigint not null,
    tournament_id bigint not null,
    -- Posição da fase (tournament_stages.position).
    stage         smallint not null,
    -- Rodada, de 1 ao teto do engine (swiss::MAX_ROUNDS).
    round         smallint not null constraint stage_pairings_round check (round between 1 and 127),
    -- A partida sw-<round>-<number>: os pares de 1 em diante e, por último, o
    -- bye. Até 64 por rodada (128 participantes, swiss::MAX_PARTICIPANTS).
    number        smallint not null constraint stage_pairings_number check (number between 1 and 64),
    -- O lado a (o melhor colocado na tabela do pareamento) e o b; nulo: o
    -- bye de side_a.
    side_a        bigint not null,
    side_b        bigint,
    created_at    timestamptz not null default now(),
    primary key (tournament_id, stage, round, number),
    foreign key (tenant_id, tournament_id) references tournaments (tenant_id, id)
        on delete cascade,
    foreign key (tournament_id, stage) references tournament_stages (tournament_id, position),
    foreign key (tournament_id, side_a) references teams (tournament_id, id),
    foreign key (tournament_id, side_b) references teams (tournament_id, id),
    constraint stage_pairings_sides check (side_a <> side_b)
);

comment on table stage_pairings is
    'Pareamento de cada rodada do suíço, gravado quando a rodada nasce; o replay da fase usa estes.';

-- Na hora: não muda; fase do suíço; cada time uma vez por rodada.
create function stage_pairings_guard() returns trigger
language plpgsql as $$
begin
    if tg_op = 'UPDATE' then
        raise exception 'pareamento % da rodada % não muda', old.number, old.round
            using errcode = 'check_violation', constraint = 'stage_pairings_immutable';
    end if;
    if not exists (select 1 from tournament_stages
                   where tournament_id = new.tournament_id and position = new.stage
                     and config ->> 'format' = 'swiss') then
        raise exception 'pareamento gravado numa fase que não é suíço (%)', new.tournament_id
            using errcode = 'check_violation', constraint = 'stage_pairings_swiss';
    end if;
    if exists (select 1 from stage_pairings
               where tournament_id = new.tournament_id and stage = new.stage
                 and round = new.round
                 and (side_a in (new.side_a, new.side_b) or side_b in (new.side_a, new.side_b)))
    then
        raise exception 'time pareado duas vezes na rodada %', new.round
            using errcode = 'check_violation', constraint = 'stage_pairings_team_once';
    end if;
    return new;
end $$;

create trigger stage_pairings_guard
    before insert or update on stage_pairings
    for each row execute function stage_pairings_guard();

-- No commit: só em torneio iniciado (a primeira rodada entra no "Iniciar",
-- antes de o status mudar na mesma transação); cada rodada com as partidas
-- de 1 a n, o bye (se houver) na última, e a anterior já gravada.
create function stage_pairings_valid() returns trigger
language plpgsql as $$
declare
    total integer;
    last_number integer;
    bye_number integer;
begin
    if not exists (select 1 from stage_pairings
                   where tournament_id = new.tournament_id and stage = new.stage
                     and round = new.round and number = new.number) then
        return null;
    end if;
    if not tournament_started(new.tournament_id) then
        raise exception 'pareamento em torneio não iniciado (%)', new.tournament_id
            using errcode = 'check_violation', constraint = 'stage_pairings_started';
    end if;
    select count(*), max(number), max(number) filter (where side_b is null)
    into total, last_number, bye_number
    from stage_pairings
    where tournament_id = new.tournament_id and stage = new.stage and round = new.round;
    if total <> last_number
       or (bye_number is not null and bye_number <> last_number)
       or (total = 1 and bye_number is not null)
       or (new.round > 1 and not exists (
               select 1 from stage_pairings
               where tournament_id = new.tournament_id and stage = new.stage
                 and round = new.round - 1))
    then
        raise exception 'rodada % do suíço fora da sequência', new.round
            using errcode = 'check_violation', constraint = 'stage_pairings_sequence';
    end if;
    return null;
end $$;

create constraint trigger stage_pairings_valid
    after insert on stage_pairings
    deferrable initially deferred
    for each row execute function stage_pairings_valid();

-- Pareamento de torneio iniciado não sai avulso (só com o torneio ou ao
-- desfazer o início, quando no commit ele não está mais iniciado).
create function stage_pairings_delete() returns trigger
language plpgsql as $$
begin
    if tournament_started(old.tournament_id) then
        raise exception 'pareamento do torneio iniciado % não sai', old.tournament_id
            using errcode = 'foreign_key_violation', constraint = 'stage_pairings_started_delete';
    end if;
    return null;
end $$;

create constraint trigger stage_pairings_delete
    after delete on stage_pairings
    deferrable initially deferred
    for each row execute function stage_pairings_delete();

-- Torneio que volta a não ter começado (desfazer o início) não fica com
-- pareamento.
create function tournaments_unstarted_pairings() returns trigger
language plpgsql as $$
begin
    if exists (select 1 from tournaments where id = new.id)
       and not tournament_started(new.id)
       and exists (select 1 from stage_pairings where tournament_id = new.id)
    then
        raise exception 'torneio % não começou e tem pareamento', new.id
            using errcode = 'check_violation', constraint = 'stage_pairings_started';
    end if;
    return null;
end $$;

create constraint trigger tournaments_unstarted_pairings
    after update of status on tournaments
    deferrable initially deferred
    for each row
    when (old.status in ('in_progress', 'finished') and new.status not in ('in_progress', 'finished'))
    execute function tournaments_unstarted_pairings();
