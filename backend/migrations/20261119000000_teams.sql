-- Times em todos os modos (decisões em "Formatos" e "Rating" no
-- docs/decisions.md). Quem ocupa um lado da chave passa a ser um time: no
-- duelo, um time de um jogador, que nasce com a inscrição e some com ela. A
-- inscrição continua individual (participants, por user_id), agora com o time
-- do inscrito. Cada mapa grava quem jogou por lado (match_game_players), a
-- fonte do rating, que passa a ser um só por comunidade, para todos os modos.
--
-- Decisão do usuário (29/10): a produção ainda não tem usuários, então os
-- torneios são apagados em vez de convertidos. Contas, comunidades, membros,
-- convites de comunidade, configurações e ajustes de rating ficam; os links de
-- inscrição em torneio (tenant_invites com tournament_id) vão com os torneios,
-- e quem entrou na comunidade por um deles fica sem o link de entrada
-- (tenant_members.invite_id nulo, como num link apagado).

-- 1. Apagar os torneios. A cascata leva inscrições, fases, mapas do pool,
-- traduções, links de inscrição, séries, mapas, histórico, veto e screenshots
-- (os objetos no bucket entram na fila de exclusão pelo trigger de
-- match_screenshots). Os triggers de recusa reconhecem a cascata pelo torneio
-- já apagado. As conferências do commit que a cascata deixou pendentes rodam
-- já (o Postgres não altera tabela com evento de trigger pendente).
delete from tournaments;
set constraints all immediate;

-- 2. O rating é refeito do zero: o cache sai e toda comunidade fica marcada
-- (sobram os ajustes).
delete from rating_history;
delete from ratings;
insert into rating_dirty (tenant_id) select id from tenants;

-- 3. As referências a participants como lado da partida saem antes de mudar
-- a chave dele (as tabelas estão vazias).
alter table match_results
    drop constraint match_results_tournament_id_side_a_fkey,
    drop constraint match_results_tournament_id_side_b_fkey;
alter table match_games
    drop constraint match_games_tournament_id_side_a_fkey,
    drop constraint match_games_tournament_id_side_b_fkey;
alter table match_events drop constraint match_events_tournament_id_winner_fkey;
alter table match_veto_events
    drop constraint match_veto_events_tournament_id_side_a_fkey,
    drop constraint match_veto_events_tournament_id_side_b_fkey;

-- 4. participants: a inscrição individual, com chave natural (torneio,
-- usuário). Seed e desistência passam para o time.
drop trigger participants_withdrawal on participants;
drop function participants_withdrawal();
alter table participants
    drop constraint participants_withdrawn_seeded,
    drop constraint participants_withdrawn_by_check,
    drop constraint participants_withdrawn_by_fkey,
    drop constraint participants_seed_check,
    drop constraint participants_tournament_id_seed_key,
    drop column withdrawn_at,
    drop column withdrawn_by,
    drop column seed,
    drop constraint participants_tournament_id_user_id_key,
    drop constraint participants_tournament_id_id_key,
    drop constraint participants_pkey,
    drop column id,
    add primary key (tournament_id, user_id),
    -- O time do inscrito (nulo: sem time; no duelo, nunca). Quando o time é
    -- apagado, o inscrito fica sem time.
    add column team_id bigint;

-- A trava depois do início muda só porque o seed saiu (o check-in continua
-- congelado).
create or replace function participants_frozen_after_start() returns trigger
language plpgsql as $$
begin
    if tg_op = 'DELETE' then
        if tournament_started(old.tournament_id) then
            raise exception 'inscrito % de torneio iniciado', old.user_id
                using errcode = 'foreign_key_violation', constraint = 'participants_started_delete';
        end if;
        return old;
    end if;
    if tg_op = 'UPDATE'
       and (new.tenant_id, new.tournament_id, new.user_id)
           is distinct from (old.tenant_id, old.tournament_id, old.user_id)
    then
        raise exception 'torneio e membro do inscrito % não mudam', old.user_id
            using errcode = 'check_violation', constraint = 'participants_identity_immutable';
    end if;
    if tournament_started(new.tournament_id)
       and (tg_op = 'INSERT' or new.checked_in_at is distinct from old.checked_in_at)
    then
        raise exception 'torneio % já começou: inscritos e check-ins congelados',
                new.tournament_id
            using errcode = 'check_violation', constraint = 'participants_started_frozen';
    end if;
    return new;
end $$;

-- 5. teams: quem ocupa um lado da chave (o ParticipantId do engine).
create table teams (
    id            bigint generated always as identity primary key,
    tenant_id     bigint not null,
    tournament_id bigint not null,
    -- Nome do time; nulo no duelo (o time de um jogador aparece pelo apelido
    -- dele) e, nos times, enquanto a organização não der um.
    name          text check (char_length(name) between 1 and 40),
    -- O capitão: um inscrito do próprio time (no duelo, o único). Apagar a
    -- inscrição dele apaga o time (no duelo, o time some com a inscrição).
    captain       bigint not null,
    -- Posição no seeding (1 = melhor); nulo sem seed. Depois de iniciar, 1..N
    -- entre os times da chave.
    seed          integer check (seed >= 1),
    -- Quando o time desistiu (nulo: segue no torneio). Só com o torneio em
    -- andamento, só quem está na chave, e não se desfaz.
    withdrawn_at  timestamptz,
    -- Organizador que marcou (nulo: saiu da comunidade sozinho). Sai com o
    -- autor.
    withdrawn_by  bigint,
    created_at    timestamptz not null default now(),
    unique (tournament_id, id),
    unique (tournament_id, seed),
    constraint teams_withdrawn_by_check check (withdrawn_by is null or withdrawn_at is not null),
    constraint teams_withdrawn_seeded check (withdrawn_at is null or seed is not null),
    foreign key (tenant_id, tournament_id) references tournaments (tenant_id, id)
        on delete cascade,
    -- No commit: o time nasce antes da inscrição do capitão (o duelo cria o
    -- time no insert da inscrição).
    constraint teams_captain_fkey foreign key (tournament_id, captain)
        references participants (tournament_id, user_id)
        on delete cascade deferrable initially deferred,
    constraint teams_withdrawn_by_fkey foreign key (tenant_id, withdrawn_by)
        references tenant_members (tenant_id, user_id) on delete set null (withdrawn_by)
);

-- Um capitão por time, e cada inscrito capitão de um time só (ele está num
-- time só). Também serve à cascata da inscrição.
create unique index teams_captain_key on teams (tournament_id, captain);
create index teams_withdrawn_by_idx on teams (tenant_id, withdrawn_by)
    where withdrawn_by is not null;

alter table participants
    add constraint participants_team_id_fkey foreign key (tournament_id, team_id)
        references teams (tournament_id, id) on delete set null (team_id);
create index participants_team_idx on participants (tournament_id, team_id)
    where team_id is not null;

-- Torneio, tenant e id do time não mudam; depois do início, nenhum time entra
-- nem sai e o seed fica (nome e capitão ainda mudam).
create function teams_frozen_after_start() returns trigger
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
       and (tg_op = 'INSERT' or new.seed is distinct from old.seed)
    then
        raise exception 'torneio % já começou: times e seeds congelados', new.tournament_id
            using errcode = 'check_violation', constraint = 'teams_started_frozen';
    end if;
    return new;
end $$;

create trigger teams_frozen_after_start
    before insert or delete or update on teams
    for each row execute function teams_frozen_after_start();

create function teams_withdrawal() returns trigger
language plpgsql as $$
begin
    if tg_op = 'UPDATE' and old.withdrawn_at is not null
       and new.withdrawn_at is distinct from old.withdrawn_at
    then
        raise exception 'a desistência do time % não se desfaz', old.id
            using errcode = 'check_violation', constraint = 'teams_withdrawal_final';
    end if;
    if new.withdrawn_at is not null
       and (tg_op = 'INSERT' or old.withdrawn_at is null)
       and not exists (select 1 from tournaments
                       where id = new.tournament_id and status = 'in_progress')
    then
        raise exception 'desistência só com o torneio % em andamento', new.tournament_id
            using errcode = 'check_violation', constraint = 'teams_withdrawal_in_progress';
    end if;
    return new;
end $$;

create trigger teams_withdrawal
    before insert or update of withdrawn_at on teams
    for each row execute function teams_withdrawal();

-- No duelo, o time de um jogador nasce com a inscrição: quem inscreve não
-- precisa saber de time, e o seeding ordena times do mesmo jeito em todos os
-- modos. Roda depois das recusas da inscrição (os triggers before rodam em
-- ordem alfabética).
create function participants_solo_team() returns trigger
language plpgsql as $$
begin
    if new.team_id is null
       and exists (select 1 from tournaments where id = new.tournament_id and mode = 'duel')
    then
        insert into teams (tenant_id, tournament_id, captain)
        values (new.tenant_id, new.tournament_id, new.user_id)
        returning id into new.team_id;
    end if;
    return new;
end $$;

create trigger participants_solo_team
    before insert on participants
    for each row execute function participants_solo_team();

-- No commit: o capitão é do time; no duelo, o time tem exatamente um jogador
-- e nenhum nome, e todo inscrito tem time.
create function check_team(team bigint) returns void
language plpgsql as $$
declare
    t record;
begin
    select tm.id, tm.tournament_id, tm.captain, tm.name, tr.mode
      into t
      from teams tm
      join tournaments tr on tr.id = tm.tournament_id
     where tm.id = team;
    if not found then
        return;
    end if;
    if not exists (select 1 from participants
                   where tournament_id = t.tournament_id and user_id = t.captain
                     and team_id = t.id)
    then
        raise exception 'o capitão % não é do time %', t.captain, team
            using errcode = 'check_violation', constraint = 'teams_captain_member';
    end if;
    if t.mode = 'duel'
       and (t.name is not null
            or (select count(*) from participants where tournament_id = t.tournament_id
                                                    and team_id = t.id) <> 1)
    then
        raise exception 'time % do duelo sem ser de um jogador só, sem nome', team
            using errcode = 'check_violation', constraint = 'teams_duel_solo';
    end if;
end $$;

create function teams_members() returns trigger
language plpgsql as $$
begin
    perform check_team(new.id);
    return null;
end $$;

create constraint trigger teams_members
    after insert or update on teams
    deferrable initially deferred
    for each row execute function teams_members();

create function participants_team() returns trigger
language plpgsql as $$
begin
    if tg_op <> 'DELETE'
       and exists (select 1 from participants p
                   join tournaments t on t.id = p.tournament_id
                   where p.tournament_id = new.tournament_id and p.user_id = new.user_id
                     and p.team_id is null and t.mode = 'duel')
    then
        raise exception 'inscrito % do duelo sem time', new.user_id
            using errcode = 'check_violation', constraint = 'participants_duel_team';
    end if;
    if tg_op <> 'INSERT' and old.team_id is not null then
        perform check_team(old.team_id);
    end if;
    if tg_op <> 'DELETE' and new.team_id is not null then
        perform check_team(new.team_id);
    end if;
    return null;
end $$;

create constraint trigger participants_team
    after insert or update or delete on participants
    deferrable initially deferred
    for each row execute function participants_team();

-- Torneio que volta a não ter começado (desfazer o início): inscritos que
-- saíram da comunidade e times que desistiram saem antes do commit.
create or replace function tournaments_unstarted_participants() returns trigger
language plpgsql as $$
begin
    if exists (select 1 from tournaments where id = new.id)
       and not tournament_started(new.id)
    then
        if exists (select 1 from participants p
                   join tenant_members m on m.tenant_id = p.tenant_id and m.user_id = p.user_id
                   where p.tournament_id = new.id and m.left_at is not null)
        then
            raise exception 'torneio % não começou e tem inscrito inativo', new.id
                using errcode = 'check_violation', constraint = 'participants_member_active';
        end if;
        if exists (select 1 from teams
                   where tournament_id = new.id and withdrawn_at is not null)
        then
            raise exception 'torneio % não começou e tem desistente', new.id
                using errcode = 'check_violation', constraint = 'teams_withdrawn_started';
        end if;
    end if;
    return null;
end $$;

-- 6. Os lados da partida são times.
alter table match_results
    add constraint match_results_tournament_id_side_a_fkey foreign key (tournament_id, side_a)
        references teams (tournament_id, id),
    add constraint match_results_tournament_id_side_b_fkey foreign key (tournament_id, side_b)
        references teams (tournament_id, id);
alter table match_games
    add constraint match_games_tournament_id_side_a_fkey foreign key (tournament_id, side_a)
        references teams (tournament_id, id),
    add constraint match_games_tournament_id_side_b_fkey foreign key (tournament_id, side_b)
        references teams (tournament_id, id),
    add constraint match_games_tournament_id_id_key unique (tournament_id, id);
alter table match_events
    add constraint match_events_tournament_id_winner_fkey foreign key (tournament_id, winner)
        references teams (tournament_id, id);
alter table match_veto_events
    add constraint match_veto_events_tournament_id_side_a_fkey foreign key (tournament_id, side_a)
        references teams (tournament_id, id),
    add constraint match_veto_events_tournament_id_side_b_fkey foreign key (tournament_id, side_b)
        references teams (tournament_id, id);

-- O W.O. automático: o time perdedor desistiu ou nenhum jogador dele continua
-- na comunidade.
create or replace function match_results_withdrawal_loser() returns trigger
language plpgsql as $$
begin
    if not exists (
        select 1 from teams t
        where t.tournament_id = new.tournament_id
          and t.id = case when new.winner = new.side_a then new.side_b else new.side_a end
          and (t.withdrawn_at is not null
               or not exists (select 1 from participants p
                              join tenant_members m
                                on m.tenant_id = p.tenant_id and m.user_id = p.user_id
                              where p.tournament_id = t.tournament_id and p.team_id = t.id
                                and m.left_at is null)))
    then
        raise exception 'W.O. automático em % sem desistente', new.match_key
            using errcode = 'check_violation', constraint = 'match_results_withdrawal_loser';
    end if;
    return new;
end $$;

-- 7. A escalação de cada mapa: quem jogou por lado. É a fonte do rating (nos
-- times haverá substituição entre mapas). Gravada pelo banco quando o mapa
-- entra ou muda de lados: os inscritos de cada time naquele momento (no
-- duelo, o próprio jogador). Não muda depois.
create table match_game_players (
    tenant_id     bigint not null,
    tournament_id bigint not null,
    game_id       bigint not null,
    side          text not null check (side in ('a', 'b')),
    user_id       bigint not null,
    primary key (game_id, user_id),
    foreign key (tenant_id, tournament_id) references tournaments (tenant_id, id)
        on delete cascade,
    foreign key (tournament_id, game_id) references match_games (tournament_id, id)
        on delete cascade,
    -- Inscrito de torneio iniciado não sai avulso; a cascata do torneio passa.
    foreign key (tournament_id, user_id) references participants (tournament_id, user_id)
);

create index match_game_players_user_idx on match_game_players (tournament_id, user_id);

create function match_games_lineup() returns trigger
language plpgsql as $$
begin
    if tg_op = 'UPDATE' then
        if (new.side_a, new.side_b) = (old.side_a, old.side_b) then
            return null;
        end if;
        delete from match_game_players where game_id = new.id;
    end if;
    insert into match_game_players (tenant_id, tournament_id, game_id, side, user_id)
    select new.tenant_id, new.tournament_id, new.id, s.side, p.user_id
    from (values ('a', new.side_a), ('b', new.side_b)) as s (side, team)
    join participants p on p.tournament_id = new.tournament_id and p.team_id = s.team;
    return null;
end $$;

create trigger match_games_lineup
    after insert or update of side_a, side_b on match_games
    for each row execute function match_games_lineup();

-- Quem entra na escalação é do time daquele lado agora; a linha não muda.
create function match_game_players_guard() returns trigger
language plpgsql as $$
begin
    if tg_op = 'UPDATE' then
        raise exception 'a escalação do mapa % não muda', old.game_id
            using errcode = 'check_violation', constraint = 'match_game_players_immutable';
    end if;
    if not exists (select 1 from match_games g
                   join participants p
                     on p.tournament_id = g.tournament_id and p.user_id = new.user_id
                   where g.id = new.game_id and g.tournament_id = new.tournament_id
                     and p.team_id = case new.side when 'a' then g.side_a else g.side_b end)
    then
        raise exception 'jogador % fora do time do lado % do mapa %', new.user_id, new.side,
                new.game_id
            using errcode = 'check_violation', constraint = 'match_game_players_team';
    end if;
    return new;
end $$;

create trigger match_game_players_guard
    before insert or update on match_game_players
    for each row execute function match_game_players_guard();

-- No commit: todo mapa tem alguém de cada lado.
create function check_game_lineup(game bigint) returns void
language plpgsql as $$
begin
    if exists (select 1 from match_games g where g.id = game)
       and (select count(distinct side) from match_game_players where game_id = game) <> 2
    then
        raise exception 'mapa % sem escalação dos dois lados', game
            using errcode = 'check_violation', constraint = 'match_game_players_required';
    end if;
end $$;

create function match_games_lineup_required() returns trigger
language plpgsql as $$
begin
    if tg_table_name = 'match_games' then
        perform check_game_lineup(new.id);
    else
        perform check_game_lineup(old.game_id);
    end if;
    return null;
end $$;

create constraint trigger match_games_lineup_required
    after insert or update on match_games
    deferrable initially deferred
    for each row execute function match_games_lineup_required();

create constraint trigger match_game_players_required
    after delete on match_game_players
    deferrable initially deferred
    for each row execute function match_games_lineup_required();

-- A escalação de um mapa confirmado é fonte do rating.
create function match_game_players_rating_dirty() returns trigger
language plpgsql as $$
begin
    if tg_op = 'DELETE' then
        if exists (select 1 from match_games where id = old.game_id and status = 'confirmed') then
            perform rating_mark_dirty(old.tenant_id);
        end if;
    elsif exists (select 1 from match_games where id = new.game_id and status = 'confirmed') then
        perform rating_mark_dirty(new.tenant_id);
    end if;
    return null;
end $$;

create trigger match_game_players_rating_dirty
    after insert or delete on match_game_players
    for each row execute function match_game_players_rating_dirty();

-- 8. Um ranking só por comunidade, para todos os modos: a modalidade sai.
-- O adversário de cada mapa no histórico sai da escalação do mapa
-- (match_game_players), então rating_history não o guarda mais.
drop index rating_adjustments_order_idx;
alter table rating_adjustments drop column modality;
create index rating_adjustments_order_idx on rating_adjustments (tenant_id, created_at, id);

alter table ratings
    drop constraint ratings_pkey,
    drop column modality,
    add primary key (tenant_id, user_id);

alter table rating_history
    drop constraint rating_history_pkey,
    drop constraint rating_history_game,
    drop constraint rating_history_opponent,
    drop column modality,
    drop column opponent_id,
    add primary key (tenant_id, user_id, seq),
    add constraint rating_history_game check ((game_id is null) = (won is null));
