-- Série mapa a mapa (decisões em "Resultados" no docs/decisions.md). Cada
-- mapa (jogo) da série é enviado e confirmado sozinho, em match_games; a
-- série fecha quando os mapas confirmados, em sequência desde o primeiro, dão
-- a maioria a um lado, e só então entra em match_results (que passa a
-- guardar só a série decidida: pelos mapas, pelo organizador ou pelo W.O.
-- automático). As regras da série são do engine (series::games) e se repetem
-- aqui como invariantes.
--
-- Dados existentes, convertidos de forma determinística:
-- - série com placar (confirmada, pendente ou contestada) vira um mapa por
--   mapa jogado. Com detalhe por mapa, cada mapa vem do detalhe (mapa e
--   frags; o vencedor é quem fez mais frags). Sem detalhe, os mapas ficam sem
--   nome nem frags (map_id nulo), primeiro os do perdedor da série e depois
--   os do vencedor (assim o último é sempre de quem venceu);
-- - cada mapa herda o estado e a autoria do envio da série: quem enviou e
--   quando, o prazo, a contestação (com o comentário em todos os mapas da
--   série contestada) e a confirmação;
-- - a série pendente ou contestada sai de match_results: ela ainda não está
--   decidida, e fecha quando os mapas confirmarem (pelo prazo que já tinham,
--   ou pelo organizador);
-- - W.O. não tem mapas;
-- - o histórico antigo fica como está: os eventos sem mapa (game nulo) são da
--   série inteira.

-- 1. O detalhe antigo, guardado para a conversão.
create table match_games_legacy as select * from match_games;
drop table match_games;

-- 2. Mapas da série: um por mapa jogado, na ordem.
create table match_games (
    id              bigint generated always as identity primary key,
    tenant_id       bigint not null,
    tournament_id   bigint not null,
    -- Posição da fase (tournament_stages.position).
    stage           smallint not null,
    -- engine::core::MatchId legível: wb-1-3, lb-2-1, gf-1-1.
    match_key       text not null check (match_key ~ '^(wb|lb|gf)-[1-9][0-9]*-[1-9][0-9]*$'),
    -- Número do mapa na série, a partir de 1, sem buracos.
    number          smallint not null check (number between 1 and 255),
    -- Lados da partida quando o mapa entrou (iguais em todos os mapas dela).
    side_a          bigint not null,
    side_b          bigint not null,
    winner          bigint not null,
    -- Nulo só em série antiga enviada sem detalhe por mapa.
    map_id          bigint references maps (id),
    -- Frags de cada lado; opcionais.
    a_frags         smallint check (a_frags between 0 and 999),
    b_frags         smallint check (b_frags between 0 and 999),
    status          text not null check (status in ('pending', 'contested', 'confirmed')),
    -- Lado do jogador que enviou; nulo quando o organizador lançou.
    reported_side   text check (reported_side in ('a', 'b')),
    -- Quem enviou ou lançou o que vale agora.
    reported_by     bigint references users (id) on delete set null,
    reported_at     timestamptz not null default now(),
    -- Fim da janela de contestação do mapa enviado por um jogador.
    confirm_by      timestamptz,
    contested_by    bigint references users (id) on delete set null,
    contested_at    timestamptz,
    contest_comment text check (length(contest_comment) between 1 and 280),
    confirmed_via   text check (confirmed_via in ('opponent', 'deadline', 'organizer')),
    -- Nulo na confirmação por tempo.
    confirmed_by    bigint references users (id) on delete set null,
    confirmed_at    timestamptz,
    unique (tournament_id, stage, match_key, number),
    -- O mapa não se repete na série (os nulos, antigos, não contam).
    unique (tournament_id, stage, match_key, map_id),
    foreign key (tenant_id, tournament_id) references tournaments (tenant_id, id)
        on delete cascade,
    foreign key (tournament_id, stage) references tournament_stages (tournament_id, position),
    foreign key (tournament_id, side_a) references participants (tournament_id, id),
    foreign key (tournament_id, side_b) references participants (tournament_id, id),
    constraint match_games_sides check (side_a <> side_b),
    constraint match_games_winner check (winner in (side_a, side_b)),
    -- Frags: os dois ou nenhum, só com o mapa, sem empate (o duelo sempre
    -- tem vencedor) e com mais frags para quem venceu.
    constraint match_games_frags check ((a_frags is null) = (b_frags is null)),
    constraint match_games_frags_map check (a_frags is null or map_id is not null),
    constraint match_games_frags_winner
        check (a_frags is null or (a_frags <> b_frags and (winner = side_a) = (a_frags > b_frags))),
    -- Enviado por um jogador: tem lado e prazo. Lançado pelo organizador:
    -- nasce confirmado por ele.
    constraint match_games_reported check ((reported_side is null) = (confirm_by is null)),
    constraint match_games_window check (confirm_by is null or confirm_by > reported_at),
    constraint match_games_entered check (reported_side is not null or confirmed_via = 'organizer'),
    -- Contestado: quando e por quê; só enquanto espera o organizador.
    constraint match_games_contested check ((status = 'contested') = (contested_at is not null)),
    constraint match_games_contest_comment
        check ((contested_at is null) = (contest_comment is null)),
    constraint match_games_contested_by check (contested_by is null or contested_at is not null),
    -- Confirmado: como e quando.
    constraint match_games_confirmed check ((status = 'confirmed') = (confirmed_via is not null)),
    constraint match_games_confirmed_at check ((status = 'confirmed') = (confirmed_at is not null)),
    constraint match_games_deadline
        check (confirmed_via <> 'deadline' or (confirmed_by is null and confirmed_at = confirm_by)),
    constraint match_games_opponent check (confirmed_via <> 'opponent' or reported_side is not null)
);

-- 3. Conversão das séries com placar.
insert into match_games
    (tenant_id, tournament_id, stage, match_key, number, side_a, side_b, winner, map_id,
     a_frags, b_frags, status, reported_side, reported_by, reported_at, confirm_by,
     contested_by, contested_at, contest_comment, confirmed_via, confirmed_by, confirmed_at)
select r.tenant_id, r.tournament_id, r.stage, r.match_key, n.number::smallint, r.side_a, r.side_b,
       case
           when g.a_frags is not null
               then case when g.a_frags > g.b_frags then r.side_a else r.side_b end
           when n.number <= l.loser_maps then l.loser
           else r.winner
       end,
       g.map_id, g.a_frags, g.b_frags, r.status, r.reported_side, r.reported_by, r.reported_at,
       r.confirm_by, r.contested_by, r.contested_at, r.contest_comment, r.confirmed_via,
       r.confirmed_by, r.confirmed_at
from match_results r
cross join lateral (
    select case when r.winner = r.side_a then r.side_b else r.side_a end as loser,
           case when r.winner = r.side_a then r.b_wins else r.a_wins end as loser_maps
) l
cross join lateral generate_series(1, r.a_wins + r.b_wins) as n(number)
left join match_games_legacy g on g.result_id = r.id and g.number = n.number
where r.a_wins is not null;

drop table match_games_legacy;

-- A série pendente ou contestada ainda não está decidida: fica só nos mapas.
delete from match_results where status <> 'confirmed';

create index match_games_due_idx on match_games (tournament_id, confirm_by)
    where status = 'pending';

-- 4. match_results: só a série decidida. Pelos mapas ('maps'): o placar
-- deles, sem quem enviou nem quem confirmou (cada mapa tem os seus).
alter table match_results
    add constraint match_results_decided check (status = 'confirmed'),
    drop constraint match_results_confirmed_via_check,
    add constraint match_results_confirmed_via_check
        check (confirmed_via in ('opponent', 'deadline', 'organizer', 'withdrawal', 'maps')),
    drop constraint match_results_reported_check,
    add constraint match_results_reported_check
        check (reported_side is not null or confirmed_via in ('organizer', 'withdrawal', 'maps')),
    add constraint match_results_maps_check
        check (confirmed_via <> 'maps'
               or (a_wins is not null and reported_side is null
                   and reported_by is null and confirmed_by is null));

-- 5. Série e mapas batem, conferido no commit (cada ação grava mapas e série
-- em comandos separados): os mapas de uma partida têm os mesmos lados e
-- números de 1 a n; com a série decidida, todos estão confirmados, com os
-- lados dela, e o placar dela (se não foi W.O.) é o dos mapas.
create function check_match_series(t bigint, s smallint, k text) returns void
language plpgsql as $$
declare
    r match_results;
    a_won bigint;
    b_won bigint;
begin
    if exists (select 1 from match_games
               where tournament_id = t and stage = s and match_key = k
               having count(distinct (side_a, side_b)) > 1 or max(number) <> count(*))
    then
        raise exception 'mapas da partida % com lados diferentes ou números fora de ordem', k
            using errcode = 'check_violation', constraint = 'match_games_sequence';
    end if;
    select * into r from match_results
    where tournament_id = t and stage = s and match_key = k;
    if not found then
        return;
    end if;
    if exists (select 1 from match_games
               where tournament_id = t and stage = s and match_key = k
                 and (status <> 'confirmed' or side_a <> r.side_a or side_b <> r.side_b))
    then
        raise exception 'série % decidida com mapa sem confirmar ou de outros lados', k
            using errcode = 'check_violation', constraint = 'match_games_series_decided';
    end if;
    if r.a_wins is null then
        return;
    end if;
    select count(*) filter (where winner = r.side_a), count(*) filter (where winner = r.side_b)
    into a_won, b_won
    from match_games
    where tournament_id = t and stage = s and match_key = k;
    if (r.a_wins, r.b_wins) is distinct from (a_won, b_won) then
        raise exception 'placar da série % diferente do dos mapas', k
            using errcode = 'check_violation', constraint = 'match_results_games_score';
    end if;
end $$;

create function match_series_changed() returns trigger
language plpgsql as $$
begin
    if tg_op <> 'INSERT' then
        perform check_match_series(old.tournament_id, old.stage, old.match_key);
    end if;
    if tg_op <> 'DELETE' then
        perform check_match_series(new.tournament_id, new.stage, new.match_key);
    end if;
    return null;
end $$;

create constraint trigger match_games_series
    after insert or update or delete on match_games
    deferrable initially deferred
    for each row execute function match_series_changed();

create constraint trigger match_results_series
    after insert or update on match_results
    deferrable initially deferred
    for each row execute function match_series_changed();

-- 6. Histórico: o mapa a que o evento se refere (nulo: a série inteira).
-- Evento de mapa grava só o vencedor do mapa, nas ações que o definem;
-- 'decided' é a série fechada pelos mapas (vencedor e placar, sem autor) e
-- 'reopened', a correção que a deixou sem decisão.
alter table match_events
    add column game smallint check (game between 1 and 255),
    drop constraint match_events_kind_check,
    add constraint match_events_kind_check check (kind in (
        'reported', 'confirmed', 'auto_confirmed', 'contested', 'resolved',
        'entered', 'corrected', 'walkover', 'auto_walkover', 'voided', 'decided', 'reopened')),
    drop constraint match_events_outcome_check,
    add constraint match_events_outcome_check check (case
        when game is not null then
            kind in ('reported', 'confirmed', 'auto_confirmed', 'contested', 'resolved',
                     'entered', 'corrected', 'voided')
            and winner_maps is null
            and (winner is not null) = (kind in ('reported', 'resolved', 'entered', 'corrected'))
        when kind in ('reported', 'resolved', 'entered', 'corrected', 'decided')
            then winner is not null and winner_maps is not null
        when kind in ('walkover', 'auto_walkover') then winner is not null and winner_maps is null
        else winner is null and winner_maps is null
    end),
    add constraint match_events_decided_actor check (kind <> 'decided' or actor is null);
