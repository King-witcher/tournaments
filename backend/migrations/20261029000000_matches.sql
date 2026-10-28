-- Chave e resultados (decisões em "Resultados" no docs/decisions.md). A chave
-- nunca é gravada: sai do replay da fase (config em tournament_stages,
-- participantes pelo seed) com os resultados confirmados, na ordem de
-- match_results.position. As regras de série (placar dentro do melhor de N,
-- mapa sem empate e sem repetir) são do engine e se repetem aqui como
-- invariantes.

-- Mapas do jogo, globais (como champions). Mapa novo = linha nova; o que sai
-- do jogo fica com active = false (nunca se apaga: resultados apontam para
-- ele). duel_pool marca o pool de duelo (lista provisória, o usuário revisa);
-- o veto vai ter um pool por torneio a partir daqui. O nome é nome próprio:
-- não se traduz.
create table maps (
    id        bigint generated always as identity primary key,
    slug      text not null unique check (slug ~ '^[a-z0-9]+(-[a-z0-9]+)*$'),
    name      text not null unique check (length(name) between 1 and 64),
    duel_pool boolean not null default false,
    active    boolean not null default true
);

insert into maps (slug, name, duel_pool) values
    ('awoken', 'Awoken', true),
    ('blood-covenant', 'Blood Covenant', true),
    ('blood-run', 'Blood Run', true),
    ('corrupted-keep', 'Corrupted Keep', true),
    ('deep-embrace', 'Deep Embrace', true),
    ('exile', 'Exile', true),
    ('insomnia', 'Insomnia', true),
    ('molten-falls', 'Molten Falls', true),
    ('ruins-of-sarnath', 'Ruins of Sarnath', true),
    ('vale-of-pnath', 'Vale of Pnath', true),
    ('burial-chamber', 'Burial Chamber', false),
    ('church-of-azathoth', 'Church of Azathoth', false),
    ('lockbox', 'Lockbox', false),
    ('tempest-shrine', 'Tempest Shrine', false),
    ('the-longest-yard', 'The Longest Yard', false),
    ('tower-of-koth', 'Tower of Koth', false);

-- Minutos que o adversário tem para confirmar ou contestar um resultado
-- enviado; sem resposta, confirma sozinho (conferido pela API ao ler ou agir).
alter table tournaments
    add column contest_minutes integer not null default 5
        check (contest_minutes between 1 and 1440);

-- Alvo das FKs compostas de resultados e histórico: o participante é sempre
-- do mesmo torneio.
alter table participants add unique (tournament_id, id);

-- Resultado vigente de uma partida da chave (no máximo um por partida):
-- pendente (enviado por um jogador, esperando o adversário), contestado
-- (esperando o organizador) ou confirmado (vale na chave).
--
-- Participantes e fases não podem sair enquanto houver resultado (FKs sem
-- cascade): a chave depende deles. Excluir o torneio leva tudo junto.
create table match_results (
    id              bigint generated always as identity primary key,
    tenant_id       bigint not null,
    tournament_id   bigint not null,
    -- Posição da fase (tournament_stages.position).
    stage           smallint not null,
    -- engine::core::MatchId legível: wb-1-3, lb-2-1, gf-1-1.
    match_key       text not null check (match_key ~ '^(wb|lb|gf)-[1-9][0-9]*-[1-9][0-9]*$'),
    -- Melhor de N da partida.
    best_of         smallint not null check (best_of between 1 and 255 and best_of % 2 = 1),
    -- Lados da partida (participants.id) quando o resultado entrou; o replay
    -- confere que continuam os mesmos.
    side_a          bigint not null,
    side_b          bigint not null,
    status          text not null check (status in ('pending', 'contested', 'confirmed')),
    winner          bigint not null,
    -- Mapas vencidos por cada lado; nulos no W.O.
    a_wins          smallint,
    b_wins          smallint,
    -- Lado do jogador que enviou; nulo quando o organizador lançou.
    reported_side   text check (reported_side in ('a', 'b')),
    -- Quem enviou ou lançou o placar que vale agora.
    reported_by     bigint references users (id) on delete set null,
    reported_at     timestamptz not null default now(),
    -- Fim da janela de contestação do resultado enviado por um jogador.
    confirm_by      timestamptz,
    contested_by    bigint references users (id) on delete set null,
    contested_at    timestamptz,
    contest_comment text check (length(contest_comment) between 1 and 280),
    confirmed_via   text check (confirmed_via in ('opponent', 'deadline', 'organizer')),
    -- Nulo na confirmação por tempo.
    confirmed_by    bigint references users (id) on delete set null,
    confirmed_at    timestamptz,
    -- Ordem no histórico efetivo (replay): definida ao confirmar e mantida
    -- nas correções.
    position        integer check (position >= 1),
    unique (tournament_id, stage, match_key),
    unique (tournament_id, position),
    foreign key (tenant_id, tournament_id) references tournaments (tenant_id, id)
        on delete cascade,
    foreign key (tournament_id, stage) references tournament_stages (tournament_id, position),
    foreign key (tournament_id, side_a) references participants (tournament_id, id),
    foreign key (tournament_id, side_b) references participants (tournament_id, id),
    check (side_a <> side_b),
    check (winner in (side_a, side_b)),
    -- Série jogada: o vencedor tem os mapas que o melhor de N pede, o
    -- perdedor menos que isso. W.O.: sem placar.
    check ((a_wins is null) = (b_wins is null)),
    check (a_wins is null or (
        least(a_wins, b_wins) >= 0
        and greatest(a_wins, b_wins) = best_of / 2 + 1
        and least(a_wins, b_wins) < best_of / 2 + 1
        and (winner = side_a) = (a_wins > b_wins))),
    -- Enviado por um jogador: tem lado e prazo. Lançado pelo organizador:
    -- nasce confirmado por ele.
    check ((reported_side is null) = (confirm_by is null)),
    check (confirm_by is null or confirm_by > reported_at),
    check (reported_side is not null or confirmed_via = 'organizer'),
    -- Contestado: quando e por quê; só enquanto espera o organizador.
    check ((status = 'contested') = (contested_at is not null)),
    check ((contested_at is null) = (contest_comment is null)),
    check (contested_by is null or contested_at is not null),
    -- Confirmado: como, quando e em que posição do replay.
    check ((status = 'confirmed') = (confirmed_via is not null)),
    check ((status = 'confirmed') = (confirmed_at is not null)),
    check ((status = 'confirmed') = (position is not null)),
    check (confirmed_via <> 'deadline' or (confirmed_by is null and confirmed_at = confirm_by)),
    check (confirmed_via <> 'opponent' or reported_side is not null)
);

-- Detalhe opcional por mapa de um resultado, na ordem jogada (o rating vai
-- ser por mapa).
create table match_games (
    result_id bigint not null references match_results (id) on delete cascade,
    number    smallint not null check (number between 1 and 255),
    map_id    bigint not null references maps (id),
    a_frags   smallint not null check (a_frags between 0 and 999),
    b_frags   smallint not null check (b_frags between 0 and 999),
    primary key (result_id, number),
    unique (result_id, map_id),
    -- Duelo não termina empatado.
    check (a_frags <> b_frags)
);

-- Histórico de cada partida: quem fez o quê e quando. Fica quando o resultado
-- muda (correção) ou sai (anulado); só desfazer o início apaga.
create table match_events (
    id            bigint generated always as identity primary key,
    tenant_id     bigint not null,
    tournament_id bigint not null,
    stage         smallint not null,
    match_key     text not null check (match_key ~ '^(wb|lb|gf)-[1-9][0-9]*-[1-9][0-9]*$'),
    kind          text not null check (kind in (
                      'reported', 'confirmed', 'auto_confirmed', 'contested', 'resolved',
                      'entered', 'corrected', 'walkover', 'voided')),
    -- Quem agiu; nulo na confirmação por tempo.
    actor         bigint references users (id) on delete set null,
    -- O resultado que a ação deixou, quando ela define um: vencedor e mapas
    -- do vencedor e do perdedor (nulos no W.O.).
    winner        bigint,
    winner_maps   smallint,
    loser_maps    smallint,
    comment       text check (length(comment) between 1 and 280),
    created_at    timestamptz not null default now(),
    foreign key (tenant_id, tournament_id) references tournaments (tenant_id, id)
        on delete cascade,
    foreign key (tournament_id, stage) references tournament_stages (tournament_id, position),
    foreign key (tournament_id, winner) references participants (tournament_id, id),
    check ((winner_maps is null) = (loser_maps is null)),
    check (winner_maps is null or (loser_maps >= 0 and winner_maps > loser_maps)),
    -- Placar nas ações que o definem, só vencedor no W.O., nada nas outras.
    check (case
        when kind in ('reported', 'resolved', 'entered', 'corrected')
            then winner is not null and winner_maps is not null
        when kind = 'walkover' then winner is not null and winner_maps is null
        else winner is null and winner_maps is null
    end),
    check (kind <> 'contested' or comment is not null),
    check (kind <> 'auto_confirmed' or actor is null)
);

create index match_events_match_idx on match_events (tournament_id, stage, match_key, created_at);
