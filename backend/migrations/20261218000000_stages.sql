-- Fases ligadas (do usuário, 06 e 07/11): um torneio tem de 1 a 4 fases, em
-- sequência. Da fase 2 em diante, entram os que jogaram a anterior, pela
-- classificação final dela: todos ou os `advance` melhores, sem os desistentes
-- e, com check-in na fase, só os que confirmaram presença (os de baixo do corte
-- são suplentes). Cada fase da 2 em diante começa pelo botão da organização,
-- com a anterior terminada; a 1 continua nascendo no "Iniciar" do torneio. A
-- configuração de cada fase muda até ela começar.
--
-- Nenhum torneio existente muda de comportamento: todos têm uma fase só, sem
-- corte nem check-in. A fase 1 dos torneios começados ganha o início do
-- torneio (ou, sem ele, a criação), e a dos encerrados, o fim dele.

-- 1. As colunas da fase: o corte e o check-in (configuração, congelados quando a
-- fase começa) e o andamento (início, abertura do check-in, fim), que a API
-- acerta ao iniciar, desfazer e encerrar cada fase.
alter table tournament_stages
    add column advance smallint,
    add column check_in boolean not null default false,
    add column started_at timestamptz,
    add column check_in_opened_at timestamptz,
    add column finished_at timestamptz;

-- Fase de torneio iniciado não mudava (o trigger recusa), e o preenchimento passa
-- pelas fases dos começados: os dois triggers ficam desligados só no update, como
-- em 20261211020000_stage_tiebreakers.sql.
alter table tournament_stages
    disable trigger tournament_stages_frozen_after_start,
    disable trigger tournament_stages_map_pool;

update tournament_stages s
set started_at = coalesce(t.started_at, t.created_at),
    finished_at = t.finished_at
from tournaments t
where t.id = s.tournament_id and s.position = 1 and t.status in ('in_progress', 'finished');

alter table tournament_stages
    enable trigger tournament_stages_frozen_after_start,
    enable trigger tournament_stages_map_pool;

alter table tournament_stages
    -- MAX_STAGES da API.
    add constraint tournament_stages_position_max check (position <= 4),
    -- De 2 ao maior teto dos formatos (a API confere o do formato da fase).
    add constraint tournament_stages_advance check (advance between 2 and 1024),
    -- A fase 1 recebe os inscritos do torneio: sem corte nem check-in próprios.
    add constraint tournament_stages_entry_first
        check (position > 1 or (advance is null and not check_in)),
    -- O check-in da fase abre pelo botão, só na fase que pede check-in.
    add constraint tournament_stages_check_in_opened check (check_in_opened_at is null or check_in),
    add constraint tournament_stages_finished_started
        check (finished_at is null or started_at is not null);

comment on column tournament_stages.advance is
    'Fases 2 em diante: quantos da fase anterior entram, pela classificação final dela (os disponíveis, de cima para baixo). Nulo: todos.';
comment on column tournament_stages.check_in is
    'Fases 2 em diante: só entra quem confirmar presença (stage_teams.checked_in_at), com o check-in aberto pela organização.';
comment on column tournament_stages.started_at is
    'Quando a fase começou (a 1, com o torneio). Nulo: ainda não.';
comment on column tournament_stages.check_in_opened_at is
    'Quando a organização abriu o check-in da fase; fecha quando ela começa. Nulo: não abriu.';
comment on column tournament_stages.finished_at is
    'Quando a fase terminou (todas as séries decididas e nenhum empate esperando a organização). Nulo: não terminou, ou uma correção a reabriu.';

-- 2. A trava passa a ser por fase. Depois do início do torneio, fase nova não
-- entra, e a fase começada (a 1, desde o início do torneio) não muda de
-- configuração nem sai; a que não começou muda e sai. As colunas de andamento
-- ficam de fora: são as que mudam quando a fase começa, termina ou volta.
create or replace function tournament_stages_frozen_after_start() returns trigger
language plpgsql as $$
begin
    if tg_op = 'DELETE' then
        if tournament_started(old.tournament_id)
           and (old.started_at is not null or old.position = 1)
        then
            raise exception 'fase % de torneio iniciado', old.id
                using errcode = 'foreign_key_violation', constraint = 'tournament_stages_started_delete';
        end if;
        return old;
    end if;
    if tg_op = 'INSERT' then
        if tournament_started(new.tournament_id) then
            raise exception 'torneio % já começou: fase nova não entra', new.tournament_id
                using errcode = 'check_violation', constraint = 'tournament_stages_started_frozen';
        end if;
        return new;
    end if;
    if tournament_started(old.tournament_id)
       and (old.started_at is not null or old.position = 1)
       and (new.tournament_id, new.position, new.format, new.best_of, new.finals_best_of,
            new.bracket, new.grand_final_reset, new.rounds, new.tiebreakers, new.final_tie,
            new.advance, new.check_in)
           is distinct from
           (old.tournament_id, old.position, old.format, old.best_of, old.finals_best_of,
            old.bracket, old.grand_final_reset, old.rounds, old.tiebreakers, old.final_tie,
            old.advance, old.check_in)
    then
        raise exception 'torneio % já começou: fase % congelada', old.tournament_id, old.position
            using errcode = 'check_violation', constraint = 'tournament_stages_started_frozen';
    end if;
    return new;
end $$;

-- 3. No commit: as fases de cada torneio são 1..n, a eliminação só na última (os
-- eliminados saem em faixas empatadas, sem onde cortar) e cada fase da 2 em
-- diante só fica começada com a anterior terminada. Com o torneio já apagado (a
-- cascata), não confere nada.
create function check_tournament_stage_sequence(tournament bigint) returns void
language plpgsql as $$
begin
    if not exists (select 1 from tournaments where id = tournament) then
        return;
    end if;
    if exists (select 1 from tournament_stages where tournament_id = tournament
               having count(*) <> max(position)) then
        raise exception 'fases do torneio % fora da sequência', tournament
            using errcode = 'check_violation', constraint = 'tournament_stages_sequence';
    end if;
    if exists (select 1 from tournament_stages s
               where s.tournament_id = tournament and s.format = 'elimination'
                 and s.position < (select max(position) from tournament_stages
                                   where tournament_id = tournament)) then
        raise exception 'eliminação antes da última fase do torneio %', tournament
            using errcode = 'check_violation', constraint = 'tournament_stages_elimination_last';
    end if;
    if exists (select 1 from tournament_stages s
               join tournament_stages p
                 on p.tournament_id = s.tournament_id and p.position = s.position - 1
               where s.tournament_id = tournament and s.started_at is not null
                 and p.finished_at is null) then
        raise exception 'fase do torneio % começada sem a anterior terminada', tournament
            using errcode = 'check_violation', constraint = 'tournament_stages_previous_finished';
    end if;
end $$;

create function tournament_stages_sequence() returns trigger
language plpgsql as $$
begin
    if tg_op <> 'INSERT' then
        perform check_tournament_stage_sequence(old.tournament_id);
    end if;
    if tg_op <> 'DELETE' then
        perform check_tournament_stage_sequence(new.tournament_id);
    end if;
    return null;
end $$;

create constraint trigger tournament_stages_sequence
    after insert or update or delete on tournament_stages
    deferrable initially deferred
    for each row execute function tournament_stages_sequence();

-- 4. Os times nas fases da 2 em diante: a presença confirmada no check-in da
-- fase e o seed, gravado quando ela começa (não nulo: entrou). A fase 1
-- continua em teams.seed. Linha só existe com um dos dois.
create table stage_teams (
    tenant_id      bigint not null,
    tournament_id  bigint not null,
    -- Posição da fase (tournament_stages.position), da 2 em diante.
    stage          smallint not null constraint stage_teams_stage check (stage between 2 and 4),
    team_id        bigint not null,
    -- Nulo: não confirmou presença (ou a fase não pede check-in).
    checked_in_at  timestamptz,
    -- Quem confirmou (um jogador do time ou a organização). Nulo: a conta saiu
    -- da comunidade.
    checked_in_by  bigint,
    -- De 1 a K, a ordem da classificação final da fase anterior. Nulo: não
    -- entrou, ou a fase não começou.
    seed           smallint constraint stage_teams_seed check (seed between 1 and 1024),
    primary key (tournament_id, stage, team_id),
    constraint stage_teams_seed_once unique (tournament_id, stage, seed),
    constraint stage_teams_checked_in_by check (checked_in_by is null or checked_in_at is not null),
    constraint stage_teams_present check (checked_in_at is not null or seed is not null),
    foreign key (tenant_id, tournament_id) references tournaments (tenant_id, id)
        on delete cascade,
    -- A fase que sai (só a que não começou, que só tem check-ins) leva os dela.
    foreign key (tournament_id, stage) references tournament_stages (tournament_id, position)
        on delete cascade,
    foreign key (tournament_id, team_id) references teams (tournament_id, id),
    constraint stage_teams_checked_in_by_fkey foreign key (tenant_id, checked_in_by)
        references tenant_members (tenant_id, user_id) on delete set null (checked_in_by)
);

create index stage_teams_team_idx on stage_teams (tournament_id, team_id);
create index stage_teams_checked_in_by_idx on stage_teams (tenant_id, checked_in_by)
    where checked_in_by is not null;

comment on table stage_teams is
    'Fases 2 em diante: o check-in de cada time na fase e o seed dele nela, gravado quando a fase começa.';

-- Na hora: o seed só muda com a fase por começar (a API grava os seeds antes de
-- marcar o início, e desfazer o início desmarca antes de apagá-los); a presença,
-- só com o check-in da fase aberto e ela por começar; o time é um dos que
-- jogaram a fase anterior; e linha de fase começada não sai (a cascata do
-- torneio passa: sem ele, tournament_started é falso).
create function stage_teams_guard() returns trigger
language plpgsql as $$
declare
    opened timestamptz;
    started timestamptz;
begin
    if tg_op = 'DELETE' then
        if tournament_started(old.tournament_id)
           and exists (select 1 from tournament_stages
                       where tournament_id = old.tournament_id and position = old.stage
                         and started_at is not null)
        then
            raise exception 'time % da fase % já começada', old.team_id, old.stage
                using errcode = 'foreign_key_violation', constraint = 'stage_teams_started_delete';
        end if;
        return old;
    end if;
    if tg_op = 'UPDATE'
       and (new.tenant_id, new.tournament_id, new.stage, new.team_id)
           is distinct from (old.tenant_id, old.tournament_id, old.stage, old.team_id)
    then
        raise exception 'o time e a fase da linha % não mudam', old.team_id
            using errcode = 'check_violation', constraint = 'stage_teams_identity_immutable';
    end if;
    select s.check_in_opened_at, s.started_at into opened, started
    from tournament_stages s
    where s.tournament_id = new.tournament_id and s.position = new.stage;
    if ((tg_op = 'INSERT' and new.seed is not null)
        or (tg_op = 'UPDATE' and new.seed is distinct from old.seed))
       and started is not null
    then
        raise exception 'fase % já começou: seeds congelados', new.stage
            using errcode = 'check_violation', constraint = 'stage_teams_seed_frozen';
    end if;
    if ((tg_op = 'INSERT' and new.checked_in_at is not null)
        or (tg_op = 'UPDATE' and new.checked_in_at is distinct from old.checked_in_at))
       and (started is not null or opened is null)
    then
        raise exception 'check-in da fase % fechado', new.stage
            using errcode = 'check_violation', constraint = 'stage_teams_check_in_closed';
    end if;
    if tg_op = 'INSERT' and not (
        (new.stage = 2 and exists (select 1 from teams t
                                   where t.tournament_id = new.tournament_id
                                     and t.id = new.team_id and t.seed is not null))
        or (new.stage > 2 and exists (select 1 from stage_teams p
                                      where p.tournament_id = new.tournament_id
                                        and p.stage = new.stage - 1
                                        and p.team_id = new.team_id and p.seed is not null)))
    then
        raise exception 'o time % não jogou a fase anterior à %', new.team_id, new.stage
            using errcode = 'check_violation', constraint = 'stage_teams_previous';
    end if;
    return new;
end $$;

create trigger stage_teams_guard
    before insert or update or delete on stage_teams
    for each row execute function stage_teams_guard();

-- No commit: a fase começada (da 2 em diante) tem os seeds de 1 a K, com K de
-- pelo menos 2; a que não começou, nenhum.
create function check_stage_teams(tournament bigint, stage_number smallint) returns void
language plpgsql as $$
declare
    started boolean;
    total integer;
    last_seed integer;
begin
    select s.started_at is not null into started
    from tournament_stages s
    where s.tournament_id = tournament and s.position = stage_number;
    if started is null or stage_number < 2 then
        return;
    end if;
    select count(seed), coalesce(max(seed), 0) into total, last_seed
    from stage_teams where tournament_id = tournament and stage = stage_number;
    if (started and (total < 2 or last_seed <> total)) or (not started and total > 0) then
        raise exception 'seeds da fase % do torneio % fora de 1 a n', stage_number, tournament
            using errcode = 'check_violation', constraint = 'stage_teams_seeds';
    end if;
end $$;

create function stage_teams_seeds() returns trigger
language plpgsql as $$
begin
    if tg_table_name = 'tournament_stages' then
        perform check_stage_teams(new.tournament_id, new.position);
        return null;
    end if;
    if tg_op <> 'INSERT' then
        perform check_stage_teams(old.tournament_id, old.stage);
    end if;
    if tg_op <> 'DELETE' then
        perform check_stage_teams(new.tournament_id, new.stage);
    end if;
    return null;
end $$;

create constraint trigger stage_teams_seeds
    after insert or update or delete on stage_teams
    deferrable initially deferred
    for each row execute function stage_teams_seeds();

create constraint trigger tournament_stages_seeds
    after update of started_at on tournament_stages
    deferrable initially deferred
    for each row execute function stage_teams_seeds();
