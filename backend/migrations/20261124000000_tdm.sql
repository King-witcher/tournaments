-- TDM: times de 2 a 4 jogadores (decisões em "TDM" no docs/decisions.md). A
-- fase 1 (20261119000000_teams) já fez de todo lado da chave um time; aqui
-- entram o modo, a formação pela organização, a substituição durante o
-- torneio, a escalação sem quem saiu da comunidade e a escolha simultânea de
-- campeões. Nenhum dado existente muda: todo torneio é duelo (team_size 1), o
-- JSON de champion_rules sem team_duplicates vale falso e os eventos de veto
-- ficam sem jogador.

-- 1. Modo e tamanho do time. O duelo é o time de 1; no TDM, 2 a 4.
alter table tournaments drop constraint tournaments_mode_check;
alter table tournaments
    add constraint tournaments_mode_check check (mode in ('duel', 'tdm')),
    -- Jogadores por time: 1 no duelo, de 2 a 4 no TDM.
    add column team_size smallint not null default 1
        constraint tournaments_team_size_check check (team_size between 1 and 4),
    add constraint tournaments_mode_team_size check ((mode = 'duel') = (team_size = 1)),
    -- Segundos da escolha simultânea de campeões do TDM (uma janela para
    -- todos, depois dos bans); no fim, quem não escolheu recebe um sorteado.
    add column team_pick_seconds integer not null default 60
        constraint tournaments_team_pick_seconds_check check (team_pick_seconds between 15 and 300),
    -- A seed do último sorteio dos times (o u64 do engine, bit a bit); nulo
    -- sem sorteio. Com os mesmos elegíveis (e, no balanceado, os mesmos
    -- ratings), refaz os mesmos times.
    add column teams_seed bigint;

-- 2. "Dois do mesmo time com o mesmo campeão no mesmo mapa" (TDM) entra no
-- JSON das regras de campeões; ausente vale falso (os gravados antes).
create or replace function champion_rules_valid(rules jsonb) returns boolean
language sql immutable as $$
    select coalesce(
        jsonb_typeof(rules) = 'object'
        and jsonb_typeof(rules -> 'bans_per_game') = 'number'
        and rules ->> 'bans_per_game' in ('0', '1', '2', '3')
        and case jsonb_typeof(rules -> 'no_repeat')
            when 'null' then true
            when 'object' then rules #>> '{no_repeat,who}' in ('player', 'team')
                and rules #>> '{no_repeat,scope}' in ('match', 'stage', 'tournament')
            else false
        end
        and coalesce(jsonb_typeof(rules -> 'team_duplicates'), 'boolean') = 'boolean',
        false)
$$;

-- 3. Modo, tamanho e janela da escolha congelados depois do início, como o
-- resto do formato (o veto e os times saem deles).
create or replace function tournaments_veto_frozen() returns trigger
language plpgsql as $$
begin
    if old.status in ('in_progress', 'finished') and new.status in ('in_progress', 'finished')
       and (new.map_mode, new.champion_phase, new.veto_seconds, new.team_pick_seconds)
           is distinct from (old.map_mode, old.champion_phase, old.veto_seconds,
                             old.team_pick_seconds)
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

-- 4. No commit, o time também respeita o tamanho do TDM (no máximo team_size
-- jogadores; a formação pode deixar incompleto, e iniciar exige completo,
-- conferido pela API). O nome é único no torneio, sem diferenciar maiúsculas.
create or replace function check_team(team bigint) returns void
language plpgsql as $$
declare
    t record;
    members integer;
begin
    select tm.id, tm.tournament_id, tm.captain, tm.name, tr.mode, tr.team_size
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
    members := (select count(*) from participants
                where tournament_id = t.tournament_id and team_id = t.id);
    if t.mode = 'duel' and (t.name is not null or members <> 1) then
        raise exception 'time % do duelo sem ser de um jogador só, sem nome', team
            using errcode = 'check_violation', constraint = 'teams_duel_solo';
    end if;
    if t.mode = 'tdm' and members > t.team_size then
        raise exception 'time % com % jogadores (o máximo é %)', team, members, t.team_size
            using errcode = 'check_violation', constraint = 'teams_tdm_size';
    end if;
end $$;

create unique index teams_name_key on teams (tournament_id, lower(name)) where name is not null;

-- Mudar o modo ou o tamanho (antes do início) refaz os times na mesma
-- transação, para todo caminho que mude o torneio: os times saem (os
-- inscritos ficam sem time) e, no duelo, cada inscrito ganha o time de um,
-- como na inscrição (participants_solo_team). No TDM a formação começa de
-- novo. Depois do início, o modo não muda (tournaments_mode_started_frozen).
create function tournaments_mode_teams() returns trigger
language plpgsql as $$
begin
    delete from teams where tournament_id = new.id;
    if new.mode = 'duel' then
        with created as (
            insert into teams (tenant_id, tournament_id, captain)
            select tenant_id, tournament_id, user_id
            from participants
            where tournament_id = new.id
            returning id, captain
        )
        update participants p
        set team_id = c.id
        from created c
        where p.tournament_id = new.id and p.user_id = c.captain;
    end if;
    return null;
end $$;

create trigger tournaments_mode_teams
    after update of mode, team_size on tournaments
    for each row
    when ((old.mode, old.team_size) is distinct from (new.mode, new.team_size))
    execute function tournaments_mode_teams();

-- No commit, todos os times e inscritos do torneio valem para o modo novo.
create function tournaments_teams_mode() returns trigger
language plpgsql as $$
declare
    team bigint;
begin
    if not exists (select 1 from tournaments where id = new.id) then
        return null;
    end if;
    for team in select id from teams where tournament_id = new.id loop
        perform check_team(team);
    end loop;
    if new.mode = 'duel'
       and exists (select 1 from participants where tournament_id = new.id and team_id is null)
    then
        raise exception 'inscrito do duelo % sem time', new.id
            using errcode = 'check_violation', constraint = 'participants_duel_team';
    end if;
    return null;
end $$;

create constraint trigger tournaments_teams_mode
    after update of mode, team_size on tournaments
    deferrable initially deferred
    for each row
    when ((old.mode, old.team_size) is distinct from (new.mode, new.team_size))
    execute function tournaments_teams_mode();

-- 5. Substituição no TDM em andamento: o substituto que não estava inscrito
-- entra já no time, e a inscrição dele não conta nas vagas (as vagas fecham a
-- inscrição, que já acabou). Fora isso, inscritos e check-ins continuam
-- congelados depois do início.
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
       and ((tg_op = 'INSERT'
             and not (new.team_id is not null and new.checked_in_at is null
                      and exists (select 1 from tournaments
                                  where id = new.tournament_id and mode = 'tdm')))
            or (tg_op = 'UPDATE' and new.checked_in_at is distinct from old.checked_in_at))
    then
        raise exception 'torneio % já começou: inscritos e check-ins congelados',
                new.tournament_id
            using errcode = 'check_violation', constraint = 'participants_started_frozen';
    end if;
    return new;
end $$;

create or replace function participants_capacity() returns trigger
language plpgsql as $$
begin
    if not tournament_started(new.tournament_id)
       and (select count(*) from participants where tournament_id = new.tournament_id)
           >= (select max_participants from tournaments where id = new.tournament_id)
    then
        raise exception 'torneio % lotado', new.tournament_id
            using errcode = 'check_violation', constraint = 'participants_capacity';
    end if;
    return new;
end $$;

-- 6. Quem sai do time não leva o time junto: antes de apagar a inscrição do
-- capitão de um time com outros jogadores (cancelar a inscrição antes do
-- início), a capitania passa ao jogador do time inscrito há mais tempo. Sem
-- outros, o time some com ela (a FK do capitão), como no duelo. Na cascata
-- do torneio, nada (ele já não existe).
create function participants_captain_handover() returns trigger
language plpgsql as $$
begin
    if old.team_id is not null
       and exists (select 1 from tournaments where id = old.tournament_id)
    then
        update teams t
        set captain = (select p.user_id from participants p
                       where p.tournament_id = old.tournament_id and p.team_id = old.team_id
                         and p.user_id <> old.user_id
                       order by p.registered_at, p.user_id
                       limit 1)
        where t.tournament_id = old.tournament_id and t.id = old.team_id
          and t.captain = old.user_id
          and exists (select 1 from participants p
                      where p.tournament_id = old.tournament_id and p.team_id = old.team_id
                        and p.user_id <> old.user_id);
    end if;
    return old;
end $$;

create trigger participants_captain_handover
    before delete on participants
    for each row execute function participants_captain_handover();

-- 7. A escalação de um mapa é quem está no time e na comunidade quando o mapa
-- entra (quem saiu não conta; a substituição vale a partir do mapa enviado
-- depois dela). Se ninguém do time está na comunidade (a correção de um mapa
-- antigo), entram todos, para o mapa ter os dois lados.
create or replace function match_games_lineup() returns trigger
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
    join participants p on p.tournament_id = new.tournament_id and p.team_id = s.team
    join tenant_members m on m.tenant_id = p.tenant_id and m.user_id = p.user_id
    where m.left_at is null
       or not exists (select 1 from participants o
                      join tenant_members om on om.tenant_id = o.tenant_id
                                            and om.user_id = o.user_id
                      where o.tournament_id = new.tournament_id and o.team_id = s.team
                        and om.left_at is null);
    return null;
end $$;

-- 8. A escolha simultânea de campeões do TDM: um pick de campeão com o
-- jogador. O jogador é do time daquele lado quando escolhe (depois, uma
-- substituição pode tirá-lo do time; a linha não muda).
alter table match_veto_events
    -- pick de campeão no TDM: quem escolheu (um inscrito do time do lado);
    -- nulo nos outros eventos e no duelo.
    add column player_id bigint,
    add constraint match_veto_events_player_fkey foreign key (tournament_id, player_id)
        references participants (tournament_id, user_id),
    add constraint match_veto_events_player
        check (player_id is null or (kind = 'pick' and champion_id is not null));

create index match_veto_events_player_idx on match_veto_events (tournament_id, player_id)
    where player_id is not null;

create or replace function match_veto_events_guard() returns trigger
language plpgsql as $$
declare
    begun boolean;
begin
    if tg_op = 'UPDATE' then
        raise exception 'evento de veto % não muda', old.id
            using errcode = 'check_violation', constraint = 'match_veto_events_immutable';
    end if;
    if exists (select 1 from match_veto_events
               where tournament_id = new.tournament_id and stage = new.stage
                 and match_key = new.match_key
                 and (side_a, side_b) <> (new.side_a, new.side_b)) then
        raise exception 'veto da partida % com outros lados', new.match_key
            using errcode = 'check_violation', constraint = 'match_veto_events_same_sides';
    end if;
    begun := exists (select 1 from match_veto_events
                     where tournament_id = new.tournament_id and stage = new.stage
                       and match_key = new.match_key and kind = 'begin');
    if (new.kind = 'ready' and begun) or (new.kind not in ('ready', 'begin') and not begun) then
        raise exception 'evento % fora de ordem no veto da partida %', new.kind, new.match_key
            using errcode = 'check_violation', constraint = 'match_veto_events_order';
    end if;
    if new.player_id is not null
       and not exists (select 1 from participants p
                       join tournaments t on t.id = p.tournament_id
                       where p.tournament_id = new.tournament_id and p.user_id = new.player_id
                         and t.mode = 'tdm'
                         and p.team_id = case new.side when 'a' then new.side_a
                                                       else new.side_b end)
    then
        raise exception 'jogador % fora do time do lado % no veto de %', new.player_id,
                new.side, new.match_key
            using errcode = 'check_violation', constraint = 'match_veto_events_player_team';
    end if;
    return new;
end $$;
