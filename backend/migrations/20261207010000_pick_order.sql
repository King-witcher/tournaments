-- A ordem das escolhas de campeão (decisões em docs/decisions/veto.md, "Ordem das
-- escolhas"): alternada, por jogador nos times, ou às cegas, numa janela em que
-- as escolhas de um lado ficam escondidas dos outros até ela fechar. A janela
-- aberta, que era a única nos times, sai do formulário e fica como terceiro valor
-- só para refazer o que já começou. O histórico não guarda a configuração: o
-- replay lê a linha, que os triggers congelam.
--
-- Na sala de veto, "Quem começa": o sorteio de sempre, ou o lado A.

-- 1. A coluna do torneio. O default do add column é a janela aberta de propósito:
-- os torneios em time que já começaram têm série gravada nela, e o replay precisa
-- dar o mesmo.
alter table tournaments
    add column pick_order text not null default 'open'
        constraint tournaments_pick_order_check
        check (pick_order in ('alternating', 'blind', 'open'));

comment on column tournaments.pick_order is
    'Ordem das escolhas de campeão: alternating = uma por vez, O, X, X, O..., por jogador nos times; blind = janela simultânea com as escolhas escondidas até fechar; open = janela simultânea aberta, só de torneio em time que começou antes da ordem existir.';

-- 2. O duelo fica alternado, que é idêntico ao que sempre foi, começado ou não, e
-- o que não começou passa a alternado: torneio sem início não tem evento de veto.
update tournaments
set pick_order = 'alternating'
where team_size = 1 or status not in ('in_progress', 'finished');

-- Daqui em diante o padrão é o alternado: um torneio criado pelo código antigo
-- durante o deploy, que não conhece a coluna, nasce como o da API nova.
alter table tournaments alter column pick_order set default 'alternating';

-- 3. Às cegas só existe com espelho: sem ele, os dois lados poderiam pegar o mesmo
-- campeão, e não há regra para isso.
alter table tournaments
    add constraint tournaments_blind_needs_mirror
        check (pick_order <> 'blind' or champion_rules -> 'mirror' = 'true'::jsonb);

-- 4. A ordem congela com o resto da configuração do veto depois do início (o corpo
-- é o de 20261201020000_veto_order.sql, mais a coluna nova). A linha de cima já
-- passou pelo corpo antigo, que não a olha.
create or replace function tournaments_veto_frozen() returns trigger
language plpgsql as $$
begin
    if old.status in ('in_progress', 'finished') and new.status in ('in_progress', 'finished')
       and (new.map_mode, new.champion_phase, new.veto_seconds, new.team_pick_seconds,
            new.veto_order, new.pick_order)
           is distinct from (old.map_mode, old.champion_phase, old.veto_seconds,
                             old.team_pick_seconds, old.veto_order, old.pick_order)
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

-- 5. O pick por jogador deixa de ser só do TDM: às cegas, o duelo também o tem, e
-- o jogador é do time de um do lado, como nos outros. O corpo é o de
-- 20261124000000_tdm.sql, sem exigir o modo do torneio.
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
                       where p.tournament_id = new.tournament_id and p.user_id = new.player_id
                         and p.team_id = case new.side when 'a' then new.side_a
                                                       else new.side_b end)
    then
        raise exception 'jogador % fora do time do lado % no veto de %', new.player_id,
                new.side, new.match_key
            using errcode = 'check_violation', constraint = 'match_veto_events_player_team';
    end if;
    return new;
end $$;

-- 6. A sala: a mesma coisa, com "já começou" sendo ter o evento de começo, e o
-- "Quem começa" (draw = sorteio; a = o lado A).
alter table veto_rooms
    add column pick_order text not null default 'open'
        constraint veto_rooms_pick_order
        check (pick_order in ('alternating', 'blind', 'open')),
    add column who_starts text not null default 'draw'
        constraint veto_rooms_who_starts check (who_starts in ('draw', 'a'));

update veto_rooms r
set pick_order = 'alternating'
where r.team_size = 1
   or not exists (select 1 from veto_room_events e where e.room_id = r.id and e.kind = 'begin');

alter table veto_rooms alter column pick_order set default 'alternating';

alter table veto_rooms
    add constraint veto_rooms_blind_needs_mirror check (pick_order <> 'blind' or mirror);

-- O corpo é o de 20261206000000_veto_rooms.sql, mais as colunas novas.
create or replace function veto_rooms_frozen() returns trigger
language plpgsql as $$
begin
    if (new.id, new.manage_token, new.spectator_token, new.team_size, new.best_of, new.map_mode,
        new.veto_order, new.champion_phase, new.bans_per_series, new.mirror, new.same_team,
        new.no_repeat, new.step_seconds, new.team_pick_seconds, new.created_at, new.pick_order,
        new.who_starts)
       is distinct from
       (old.id, old.manage_token, old.spectator_token, old.team_size, old.best_of, old.map_mode,
        old.veto_order, old.champion_phase, old.bans_per_series, old.mirror, old.same_team,
        old.no_repeat, old.step_seconds, old.team_pick_seconds, old.created_at, old.pick_order,
        old.who_starts) then
        raise exception 'configuração da sala de veto % não muda', old.id
            using errcode = 'check_violation', constraint = 'veto_rooms_frozen';
    end if;
    return new;
end $$;
