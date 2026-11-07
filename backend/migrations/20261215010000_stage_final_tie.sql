-- Empate no fim da fase (do usuário, 06/11): nas fases de tabela, o grupo que
-- depois de todos os critérios só a ordem da chave separa fica com ela (seed, o
-- padrão) ou espera a decisão da organização (organizer). Com organizer, no fim
-- da fase, cada grupo assim é um empate pendente até haver uma decisão para
-- exatamente aquele conjunto de times; a fase não termina antes.
--
-- As fases que já existem ficam com seed, então nenhuma tabela muda.

-- 1. A escolha da fase.
alter table tournament_stages add column final_tie text;

-- Fase de torneio iniciado não muda (o trigger recusa), e o preenchimento passa
-- por todas: os dois triggers ficam desligados só no update, como em
-- 20261211020000_stage_tiebreakers.sql.
alter table tournament_stages
    disable trigger tournament_stages_frozen_after_start,
    disable trigger tournament_stages_map_pool;

update tournament_stages set final_tie = 'seed' where format in ('round_robin', 'swiss');

alter table tournament_stages
    enable trigger tournament_stages_frozen_after_start,
    enable trigger tournament_stages_map_pool;

alter table tournament_stages
    add constraint tournament_stages_final_tie check (final_tie in ('seed', 'organizer')),
    -- O corpo é o de 20261213010000_placement.sql, com a coluna nova: presente
    -- nas fases de tabela, nula na eliminação e na disputa de posições.
    drop constraint tournament_stages_shape,
    add constraint tournament_stages_shape check (case format
        when 'elimination' then finals_best_of is not null and bracket is not null
            and (grand_final_reset is not null) = (bracket = 'double')
            and rounds is null and tiebreakers is null and final_tie is null
        when 'round_robin' then finals_best_of is null and bracket is null
            and grand_final_reset is null and rounds is null and tiebreakers is not null
            and final_tie is not null
        when 'swiss' then finals_best_of is null and bracket is null
            and grand_final_reset is null and tiebreakers is not null
            and final_tie is not null
        when 'placement' then finals_best_of is null and bracket is null
            and grand_final_reset is null and rounds is null and tiebreakers is null
            and final_tie is null
        else false
    end);

comment on column tournament_stages.final_tie is
    'Fases de tabela: quem decide o empate que sobra depois de todos os critérios, no fim da fase: seed (a ordem da chave) ou organizer (a organização). Nulo na eliminação e na disputa de posições.';

-- 2. As decisões da organização. Cada uma é a ordem de um grupo empatado, do
-- melhor ao pior (stage_tie_decision_teams). Vale enquanto o grupo empatado
-- for exatamente esse conjunto de times; uma correção que muda o conjunto a
-- deixa sem efeito, mas ela fica gravada. "Usar a ordem da chave" também grava
-- uma decisão. Só inserida (nunca muda) e apagada com o torneio ou ao desfazer
-- o início.
create table stage_tie_decisions (
    id            bigint generated always as identity primary key,
    tenant_id     bigint not null,
    tournament_id bigint not null,
    -- Posição da fase (tournament_stages.position).
    stage         smallint not null,
    -- Nulo: a conta de quem decidiu saiu da comunidade.
    decided_by    bigint,
    decided_at    timestamptz not null default now(),
    -- Opcional, até 280 caracteres numa linha (a API limpa as pontas).
    reason        text constraint stage_tie_decisions_reason
        check (length(reason) between 1 and 280),
    unique (tournament_id, id),
    foreign key (tenant_id, tournament_id) references tournaments (tenant_id, id)
        on delete cascade,
    foreign key (tournament_id, stage) references tournament_stages (tournament_id, position),
    constraint stage_tie_decisions_decided_by_fkey foreign key (tenant_id, decided_by)
        references tenant_members (tenant_id, user_id) on delete set null (decided_by)
);

create index stage_tie_decisions_stage_idx on stage_tie_decisions (tournament_id, stage);
create index stage_tie_decisions_decided_by_idx on stage_tie_decisions (tenant_id, decided_by)
    where decided_by is not null;

comment on table stage_tie_decisions is
    'Decisão da organização para um empate do fim de uma fase de tabela com final_tie = organizer; a ordem fica em stage_tie_decision_teams.';

create table stage_tie_decision_teams (
    tenant_id     bigint not null,
    tournament_id bigint not null,
    decision_id   bigint not null,
    -- De 1 (o melhor) ao tamanho do grupo; até 128 (swiss::MAX_PARTICIPANTS).
    position      smallint not null constraint stage_tie_decision_teams_position
        check (position between 1 and 128),
    team_id       bigint not null,
    primary key (decision_id, position),
    constraint stage_tie_decision_teams_team_once unique (decision_id, team_id),
    -- Direto do torneio, além da decisão: apagando o torneio, a cascata dos
    -- times confere a referência a eles antes da que viria pela decisão.
    foreign key (tenant_id, tournament_id) references tournaments (tenant_id, id)
        on delete cascade,
    foreign key (tournament_id, decision_id) references stage_tie_decisions (tournament_id, id)
        on delete cascade,
    foreign key (tournament_id, team_id) references teams (tournament_id, id)
);

create index stage_tie_decision_teams_team_idx on stage_tie_decision_teams (tournament_id, team_id);

-- Na hora: não muda; só em fase com a organização decidindo, de torneio
-- iniciado.
create function stage_tie_decisions_guard() returns trigger
language plpgsql as $$
begin
    if tg_op = 'UPDATE' then
        raise exception 'decisão de empate % não muda', old.id
            using errcode = 'check_violation', constraint = 'stage_tie_decisions_immutable';
    end if;
    if not exists (select 1 from tournament_stages
                   where tournament_id = new.tournament_id and position = new.stage
                     and final_tie = 'organizer') then
        raise exception 'decisão de empate numa fase sem a organização decidindo (%)',
                new.tournament_id
            using errcode = 'check_violation', constraint = 'stage_tie_decisions_organizer';
    end if;
    if not tournament_started(new.tournament_id) then
        raise exception 'decisão de empate em torneio não iniciado (%)', new.tournament_id
            using errcode = 'check_violation', constraint = 'stage_tie_decisions_started';
    end if;
    return new;
end $$;

create trigger stage_tie_decisions_guard
    before insert or update on stage_tie_decisions
    for each row execute function stage_tie_decisions_guard();

create function stage_tie_decision_teams_guard() returns trigger
language plpgsql as $$
begin
    raise exception 'ordem da decisão de empate % não muda', old.decision_id
        using errcode = 'check_violation', constraint = 'stage_tie_decisions_immutable';
end $$;

create trigger stage_tie_decision_teams_guard
    before update on stage_tie_decision_teams
    for each row execute function stage_tie_decision_teams_guard();

-- No commit: a ordem tem pelo menos 2 times, nas posições de 1 a n.
create function stage_tie_decisions_valid() returns trigger
language plpgsql as $$
declare
    decision bigint;
    total integer;
    last_position integer;
begin
    if tg_table_name = 'stage_tie_decisions' then
        decision := new.id;
    elsif tg_op = 'DELETE' then
        decision := old.decision_id;
    else
        decision := new.decision_id;
    end if;
    if not exists (select 1 from stage_tie_decisions where id = decision) then
        return null;
    end if;
    select count(*), max(position) into total, last_position
    from stage_tie_decision_teams where decision_id = decision;
    if total < 2 or last_position <> total then
        raise exception 'decisão de empate % sem a ordem de 1 a n', decision
            using errcode = 'check_violation', constraint = 'stage_tie_decisions_order';
    end if;
    return null;
end $$;

create constraint trigger stage_tie_decisions_valid
    after insert on stage_tie_decisions
    deferrable initially deferred
    for each row execute function stage_tie_decisions_valid();

create constraint trigger stage_tie_decision_teams_valid
    after insert or delete on stage_tie_decision_teams
    deferrable initially deferred
    for each row execute function stage_tie_decisions_valid();

-- 3. A fase que espera a organização: uma linha enquanto há empate pendente,
-- gravada quando ele aparece (é quando a organização é avisada, uma vez) e
-- apagada quando some (decidido, ou uma correção mudou a tabela). É o que a
-- home da organização lê, sem refazer a chave. Só com o torneio em andamento.
create table stage_pending_ties (
    tenant_id     bigint not null,
    tournament_id bigint not null,
    stage         smallint not null,
    since         timestamptz not null default now(),
    primary key (tournament_id, stage),
    foreign key (tenant_id, tournament_id) references tournaments (tenant_id, id)
        on delete cascade,
    foreign key (tournament_id, stage) references tournament_stages (tournament_id, position)
);

comment on table stage_pending_ties is
    'Fase de tabela com empate do fim esperando a decisão da organização (final_tie = organizer), enquanto espera.';

create function stage_pending_ties_guard() returns trigger
language plpgsql as $$
begin
    if not exists (select 1 from tournament_stages
                   where tournament_id = new.tournament_id and position = new.stage
                     and final_tie = 'organizer') then
        raise exception 'empate pendente numa fase sem a organização decidindo (%)',
                new.tournament_id
            using errcode = 'check_violation', constraint = 'stage_pending_ties_organizer';
    end if;
    return new;
end $$;

create trigger stage_pending_ties_guard
    before insert or update on stage_pending_ties
    for each row execute function stage_pending_ties_guard();

-- No commit, dos dois lados: empate pendente só em torneio em andamento.
create function stage_pending_ties_in_progress() returns trigger
language plpgsql as $$
begin
    if exists (select 1 from stage_pending_ties p
               join tournaments t on t.id = p.tournament_id
               where p.tournament_id = new.id and t.status <> 'in_progress') then
        raise exception 'torneio % com empate pendente fora do andamento', new.id
            using errcode = 'check_violation', constraint = 'stage_pending_ties_in_progress';
    end if;
    return null;
end $$;

create function stage_pending_ties_valid() returns trigger
language plpgsql as $$
begin
    if exists (select 1 from stage_pending_ties p
               join tournaments t on t.id = p.tournament_id
               where p.tournament_id = new.tournament_id and p.stage = new.stage
                 and t.status <> 'in_progress') then
        raise exception 'empate pendente fora do andamento (%)', new.tournament_id
            using errcode = 'check_violation', constraint = 'stage_pending_ties_in_progress';
    end if;
    return null;
end $$;

create constraint trigger stage_pending_ties_valid
    after insert or update on stage_pending_ties
    deferrable initially deferred
    for each row execute function stage_pending_ties_valid();

create constraint trigger tournaments_pending_ties
    after update of status on tournaments
    deferrable initially deferred
    for each row execute function stage_pending_ties_in_progress();
