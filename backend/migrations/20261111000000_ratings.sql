-- Rating (decisões em "Rating" no docs/decisions.md). O rating é derivado:
-- o resultado de reprocessar, em ordem, os mapas confirmados dos torneios
-- que contam e os ajustes manuais da organização (engine::rating). Fonte:
-- match_games, rating_adjustments, rating_settings e
-- tournaments.counts_for_rating. Cache, apagável e refeito do zero pela API
-- (ratings/refresh.rs): ratings e rating_history.
--
-- Quando a fonte muda, um trigger grava uma marca em rating_dirty, na mesma
-- transação; o agendador (e a leitura do ranking) recalcula o tenant marcado.
-- A marca é uma linha nova (nunca um update de uma linha por tenant): quem
-- grava um mapa não espera quem recalcula, e o recálculo só apaga as marcas
-- que ele viu, então a de uma transação que ainda não confirmou fica e faz
-- recalcular de novo.
--
-- Modalidade: hoje só 'duel' (o mode do torneio). As tabelas já têm a
-- coluna, para um rating por modalidade.

-- 1. O torneio conta para o rating. Padrão sim, inclusive nos que já existem;
-- a API deixa mudar até o torneio começar.
alter table tournaments
    add column counts_for_rating boolean not null default true;

-- 2. O Elo da comunidade (engine::rating::EloConfig, com as mesmas faixas).
-- Sem linha, os padrões do engine (1500, K de 80 a 32, decaimento 10).
create table rating_settings (
    tenant_id  bigint primary key references tenants (id) on delete cascade,
    initial    integer not null check (initial between 100 and 3000),
    k_min      integer not null check (k_min between 1 and 100),
    k_max      integer not null check (k_max between 1 and 200),
    k_decay    integer not null check (k_decay between 1 and 100),
    -- Quem mudou por último; nulo se o membro foi apagado.
    updated_by bigint,
    updated_at timestamptz not null default now(),
    constraint rating_settings_k_order check (k_min <= k_max),
    foreign key (tenant_id, updated_by) references tenant_members (tenant_id, user_id)
        on delete set null (updated_by)
);

-- 3. Ajuste manual: pontos a mais ou a menos para um membro, com motivo e
-- autor. Entra no reprocessamento pelo horário (created_at, depois id).
create table rating_adjustments (
    id         bigint generated always as identity primary key,
    tenant_id  bigint not null,
    modality   text not null check (modality in ('duel')),
    user_id    bigint not null,
    -- engine::rating::MAX_ADJUSTMENT.
    delta      integer not null check (delta <> 0 and delta between -1000 and 1000),
    reason     text not null check (length(reason) between 1 and 280),
    -- Quem ajustou; nulo se o membro foi apagado.
    created_by bigint,
    created_at timestamptz not null default now(),
    -- O ajuste é do membro: sai com ele (só com o tenant ou o usuário apagado).
    foreign key (tenant_id, user_id) references tenant_members (tenant_id, user_id)
        on delete cascade,
    foreign key (tenant_id, created_by) references tenant_members (tenant_id, user_id)
        on delete set null (created_by)
);

create index rating_adjustments_order_idx on rating_adjustments (tenant_id, modality, created_at, id);
create index rating_adjustments_user_idx on rating_adjustments (tenant_id, user_id);
create index rating_adjustments_created_by_idx on rating_adjustments (tenant_id, created_by);

-- 4. Cache: o rating de cada um (só quem tem jogo ou ajuste).
create table ratings (
    tenant_id    bigint not null,
    modality     text not null check (modality in ('duel')),
    user_id      bigint not null,
    rating       integer not null,
    games        integer not null check (games >= 0),
    wins         integer not null check (wins >= 0),
    losses       integer not null check (losses >= 0),
    -- Horário do último mapa que contou; nulo sem jogo (só ajustes).
    last_game_at timestamptz,
    primary key (tenant_id, modality, user_id),
    foreign key (tenant_id, user_id) references tenant_members (tenant_id, user_id)
        on delete cascade,
    constraint ratings_record check (wins + losses = games),
    constraint ratings_last_game check ((games = 0) = (last_game_at is null))
);

-- 5. Cache: o que cada evento fez com o rating de cada pessoa, na ordem do
-- reprocessamento (seq, a mesma para os dois lados de um mapa). game_id e
-- adjustment_id sem FK de propósito: a FK travaria o mapa contra o recálculo
-- (anular um mapa enquanto o histórico é regravado dá deadlock), e o mapa ou
-- o ajuste que some marca o tenant, que é recalculado antes da próxima
-- leitura.
create table rating_history (
    tenant_id     bigint not null,
    modality      text not null check (modality in ('duel')),
    user_id       bigint not null,
    seq           integer not null check (seq >= 1),
    -- match_games.id (um mapa) ou rating_adjustments.id (um ajuste).
    game_id       bigint,
    adjustment_id bigint,
    -- Nos mapas: o adversário e se venceu.
    opponent_id   bigint,
    won           boolean,
    delta         integer not null,
    rating_after  integer not null,
    -- Quando o mapa confirmou ou o ajuste foi feito.
    at            timestamptz not null,
    primary key (tenant_id, modality, user_id, seq),
    foreign key (tenant_id, user_id) references tenant_members (tenant_id, user_id)
        on delete cascade,
    foreign key (tenant_id, opponent_id) references tenant_members (tenant_id, user_id)
        on delete cascade,
    constraint rating_history_event check (num_nonnulls(game_id, adjustment_id) = 1),
    constraint rating_history_game
        check ((game_id is null) = (opponent_id is null) and (game_id is null) = (won is null)),
    constraint rating_history_opponent check (opponent_id <> user_id)
);

create index rating_history_opponent_idx on rating_history (tenant_id, opponent_id);

-- 6. As marcas de "rating desatualizado".
create table rating_dirty (
    id         bigint generated always as identity primary key,
    tenant_id  bigint not null references tenants (id) on delete cascade,
    created_at timestamptz not null default now()
);

create index rating_dirty_tenant_idx on rating_dirty (tenant_id);

-- Marca o tenant `t`, se ele ainda existe: na exclusão do tenant, a cascata
-- apaga mapas e ajustes depois dele, e a marca não teria a quem apontar.
create function rating_mark_dirty(t bigint) returns void
language sql as $$
    insert into rating_dirty (tenant_id) select id from tenants where id = t
$$;

-- Só mapa confirmado conta: enviar, contestar e anular mapa pendente não
-- marcam; confirmar, corrigir (inclusive o horário) e apagar um confirmado,
-- sim.
create function match_games_rating_dirty() returns trigger
language plpgsql as $$
begin
    if tg_op <> 'INSERT' and old.status = 'confirmed' then
        perform rating_mark_dirty(old.tenant_id);
    elsif tg_op <> 'DELETE' and new.status = 'confirmed' then
        perform rating_mark_dirty(new.tenant_id);
    end if;
    return null;
end $$;

create trigger match_games_rating_dirty
    after insert or update or delete on match_games
    for each row execute function match_games_rating_dirty();

create function rating_source_dirty() returns trigger
language plpgsql as $$
begin
    if tg_op = 'DELETE' then
        perform rating_mark_dirty(old.tenant_id);
    else
        perform rating_mark_dirty(new.tenant_id);
    end if;
    return null;
end $$;

create trigger rating_adjustments_dirty
    after insert or update or delete on rating_adjustments
    for each row execute function rating_source_dirty();

create trigger rating_settings_dirty
    after insert or update or delete on rating_settings
    for each row execute function rating_source_dirty();

create trigger tournaments_rating_dirty
    after update of counts_for_rating on tournaments
    for each row when (old.counts_for_rating is distinct from new.counts_for_rating)
    execute function rating_source_dirty();

-- 7. Os tenants que já existem calculam o primeiro ranking na primeira
-- passada do agendador.
insert into rating_dirty (tenant_id) select id from tenants;
