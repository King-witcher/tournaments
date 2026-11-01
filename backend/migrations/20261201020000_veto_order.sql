-- A ordem do veto de mapas (decisões em "Modo do mapa" no docs/decisions/veto.md).
-- O padrão passa a ser bans primeiro (os lados banem até sobrarem só os mapas
-- da série; depois escolhem a ordem dos que sobraram); o alternado (dois bans,
-- os picks, o resto dos bans) continua como opção. O histórico do veto não
-- guarda a configuração: o replay lê a linha do torneio, que o trigger
-- tournaments_veto_frozen congela depois do início.

-- 1. A coluna. O default do add column é o alternado de propósito: os
-- torneios que já começaram (em andamento ou encerrados) têm série com veto
-- gravado nessa ordem, e o replay delas precisa continuar dando o mesmo.
alter table tournaments
    add column veto_order text not null default 'alternating'
        constraint tournaments_veto_order_check check (veto_order in ('bans_first', 'alternating'));

comment on column tournaments.veto_order is
    'Ordem do veto de mapas (modo veto): bans_first = todos os bans, depois os picks; alternating = dois bans, os picks e o resto dos bans.';

-- 2. Os que ainda não começaram ganham o padrão novo. Torneio que não começou
-- não tem evento de veto (desfazer o início apaga o histórico), então nada que
-- já foi gravado muda de sentido.
update tournaments
set veto_order = 'bans_first'
where status not in ('in_progress', 'finished');

-- Daqui em diante o padrão é o novo: as linhas que existiam já receberam o
-- alternado acima, e um torneio criado pelo código antigo durante o deploy
-- (que não conhece a coluna) nasce em bans primeiro, como o da API nova.
alter table tournaments alter column veto_order set default 'bans_first';

-- 3. A ordem congela com o resto da configuração do veto depois do início (o
-- corpo é o de 20261124000000_tdm.sql, mais a coluna nova).
create or replace function tournaments_veto_frozen() returns trigger
language plpgsql as $$
begin
    if old.status in ('in_progress', 'finished') and new.status in ('in_progress', 'finished')
       and (new.map_mode, new.champion_phase, new.veto_seconds, new.team_pick_seconds,
            new.veto_order)
           is distinct from (old.map_mode, old.champion_phase, old.veto_seconds,
                             old.team_pick_seconds, old.veto_order)
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
