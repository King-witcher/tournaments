-- As regras de campeões do torneio e a configuração das fases saem do jsonb e
-- viram colunas (decisão em "Configuração em colunas" no
-- docs/decisions/database.md). Linhas antigas em JSON só falhavam ao serem
-- lidas por código novo; em coluna, o `query!` confere o SQL, um campo novo
-- exige migration e recriar um check confere as linhas gravadas.
--
-- Deploy numa etapa só, escolha do usuário (05/11): enquanto a instância nova
-- sobe, a antiga ainda lê as colunas jsonb e responde 500 nas rotas de torneio.
-- Fazer o deploy sem torneio em andamento.

-- 1. Regras de campeões. O update não dispara trigger que confira as colunas
-- novas: tournaments_veto_frozen só passa a olhar para elas no passo 3.
alter table tournaments
    add column bans_per_series smallint,
    add column mirror boolean,
    add column same_team boolean,
    add column no_repeat_who text,
    add column no_repeat_scope text;

update tournaments
set bans_per_series = (champion_rules ->> 'bans_per_series')::smallint,
    mirror = (champion_rules ->> 'mirror')::boolean,
    same_team = (champion_rules ->> 'same_team')::boolean,
    no_repeat_who = champion_rules #>> '{no_repeat,who}',
    no_repeat_scope = champion_rules #>> '{no_repeat,scope}';

alter table tournaments
    drop constraint tournaments_champion_rules_shape,
    drop constraint tournaments_blind_needs_mirror,
    drop column champion_rules,
    alter column bans_per_series set not null,
    alter column mirror set not null,
    alter column same_team set not null,
    add constraint tournaments_bans_per_series check (bans_per_series between 0 and 3),
    add constraint tournaments_no_repeat_who check (no_repeat_who in ('player', 'team')),
    add constraint tournaments_no_repeat_scope
        check (no_repeat_scope in ('series', 'stage', 'tournament')),
    add constraint tournaments_no_repeat_pair
        check ((no_repeat_who is null) = (no_repeat_scope is null)),
    add constraint tournaments_blind_needs_mirror check (pick_order <> 'blind' or mirror);

comment on column tournaments.bans_per_series is
    'Bans de campeão de cada lado na série; valem para os dois lados em todas as partidas dela.';
comment on column tournaments.mirror is
    'Adversários podem ter o mesmo campeão na mesma partida.';
comment on column tournaments.same_team is
    'Nos times: colegas podem ter o mesmo campeão na mesma partida. Sem efeito no duelo.';
comment on column tournaments.no_repeat_who is
    'Quem não repete campeão entre partidas: player ou team. Nulo: pode repetir.';
comment on column tournaments.no_repeat_scope is
    'Até onde a não repetição vale: series, stage ou tournament. Nulo: pode repetir.';

-- 2. Fases. Fase de torneio iniciado não muda (o trigger recusa), e a conversão
-- precisa passar por todas: os dois triggers ficam desligados só no update. O
-- do pool não acrescenta nada aqui e, deferido, deixaria o alter table seguinte
-- sem rodar (pending trigger events).
alter table tournament_stages
    add column format text,
    add column best_of smallint,
    add column finals_best_of smallint,
    add column bracket text,
    add column grand_final_reset boolean,
    add column rounds smallint;

alter table tournament_stages
    disable trigger tournament_stages_frozen_after_start,
    disable trigger tournament_stages_map_pool;

update tournament_stages
set format = config ->> 'format',
    best_of = coalesce(config ->> 'best_of', config #>> '{series,default}')::smallint,
    finals_best_of = (config #>> '{series,finals}')::smallint,
    bracket = config #>> '{kind,kind}',
    grand_final_reset = (config #>> '{kind,grand_final_reset}')::boolean,
    rounds = (config ->> 'rounds')::smallint;

alter table tournament_stages
    enable trigger tournament_stages_frozen_after_start,
    enable trigger tournament_stages_map_pool;

alter table tournament_stages
    drop constraint tournament_stages_config_shape,
    drop column config,
    alter column format set not null,
    alter column best_of set not null,
    add constraint tournament_stages_format
        check (format in ('elimination', 'round_robin', 'swiss')),
    add constraint tournament_stages_best_of
        check (best_of between 1 and 255 and best_of % 2 = 1),
    add constraint tournament_stages_finals_best_of
        check (finals_best_of between 1 and 255 and finals_best_of % 2 = 1),
    add constraint tournament_stages_bracket check (bracket in ('single', 'double')),
    add constraint tournament_stages_rounds check (rounds between 1 and 127),
    -- Cada formato com as suas colunas. Formato novo entra aqui também, senão
    -- o else recusa as linhas dele.
    add constraint tournament_stages_shape check (case format
        when 'elimination' then finals_best_of is not null and bracket is not null
            and (grand_final_reset is not null) = (bracket = 'double')
            and rounds is null
        when 'round_robin' then finals_best_of is null and bracket is null
            and grand_final_reset is null and rounds is null
        when 'swiss' then finals_best_of is null and bracket is null
            and grand_final_reset is null
        else false
    end);

comment on column tournament_stages.format is
    'elimination, round_robin ou swiss; diz quais das outras colunas valem (tournament_stages_shape).';
comment on column tournament_stages.best_of is
    'Melhor de N, ímpar: das partidas comuns na eliminação, de todas nas fases de tabela.';
comment on column tournament_stages.finals_best_of is
    'Melhor de N da final (simples) ou da grande final e do reset (dupla). Nulo fora da eliminação.';
comment on column tournament_stages.bracket is
    'Eliminação: single ou double. Nulo fora da eliminação.';
comment on column tournament_stages.grand_final_reset is
    'Eliminação dupla: há segunda série se o campeão da chave de baixo vence a grande final. Nulo nas outras.';
comment on column tournament_stages.rounds is
    'Suíço: rodadas, de 1 a 127. Nulo: automático (e sempre fora do suíço).';

drop function champion_rules_valid(jsonb);
drop function stage_config_valid(jsonb);
drop function best_of_valid(jsonb);

-- 3. As regras de campeões congelam com o resto da configuração depois do
-- início: as séries já começadas leem a linha atual (o corpo é o de
-- 20261207020000_team_action_rules.sql, mais as colunas novas).
create or replace function tournaments_veto_frozen() returns trigger
language plpgsql as $$
begin
    if old.status in ('in_progress', 'finished') and new.status in ('in_progress', 'finished')
       and (new.map_mode, new.champion_phase, new.veto_seconds, new.team_pick_seconds,
            new.veto_order, new.pick_order, new.veto_by, new.ban_by, new.pick_by, new.report_by,
            new.bans_per_series, new.mirror, new.same_team, new.no_repeat_who, new.no_repeat_scope)
           is distinct from (old.map_mode, old.champion_phase, old.veto_seconds,
                             old.team_pick_seconds, old.veto_order, old.pick_order,
                             old.veto_by, old.ban_by, old.pick_by, old.report_by,
                             old.bans_per_series, old.mirror, old.same_team, old.no_repeat_who,
                             old.no_repeat_scope)
    then
        raise exception 'torneio % já começou: configuração do veto congelada', old.id
            using errcode = 'check_violation', constraint = 'tournaments_veto_started_frozen';
    end if;
    if old.status in ('in_progress', 'finished') and new.status in ('in_progress', 'finished')
       and (new.mode, new.team_size) is distinct from (old.mode, old.team_size)
    then
        raise exception 'torneio % já começou: modo e tamanho do time congelados', old.id
            using errcode = 'check_violation', constraint = 'tournaments_mode_started_frozen';
    end if;
    return new;
end $$;

-- 4. O maior melhor de N vale também nas fases de tabela: a função só lia os
-- campos da eliminação e, nelas, o pool de mapas não era conferido no banco.
-- Nenhum torneio de produção ou staging fica fora da regra (conferido em 05/11).
create or replace function tournament_longest_series(tournament bigint) returns integer
language sql stable as $$
    select coalesce(max(greatest(best_of, finals_best_of)), 0)::integer
    from tournament_stages
    where tournament_id = tournament
$$;

-- 5. O corpo é o de 20261123000000_round_robin_swiss.sql, com a coluna nova.
create or replace function stage_pairings_guard() returns trigger
language plpgsql as $$
begin
    if tg_op = 'UPDATE' then
        raise exception 'pareamento % da rodada % não muda', old.number, old.round
            using errcode = 'check_violation', constraint = 'stage_pairings_immutable';
    end if;
    if not exists (select 1 from tournament_stages
                   where tournament_id = new.tournament_id and position = new.stage
                     and format = 'swiss') then
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
