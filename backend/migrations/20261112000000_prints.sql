-- Prints do placar (decisões em "Resultados" no docs/decisions.md). Cada mapa
-- da série (o número dele na partida, porque o print vem antes do envio)
-- recebe até 3 imagens, guardadas num bucket S3: aqui fica só o nome do
-- objeto. Nenhum objeto fica órfão: toda linha que sai de match_prints, por
-- qualquer caminho (removida, mapa anulado, desfazer o início, torneio ou
-- comunidade excluídos, upload nunca registrado, 30 dias depois do fim), deixa
-- o nome em print_deletions, que a rotina diária da API esvazia apagando do
-- bucket.

-- 1. O torneio decide se o envio do jogador exige print, e guarda quando
-- encerrou (os prints saem 30 dias depois).
alter table tournaments
    add column prints_required boolean not null default false,
    -- Nulo enquanto não encerrou; mantido pelo trigger tournaments_finished_at.
    add column finished_at timestamptz;

-- Dados existentes: o torneio encerrado terminou quando a última série foi
-- decidida (a final; sem série nenhuma, quando foi alterado pela última vez).
update tournaments t
set finished_at = coalesce(
    (select max(r.confirmed_at) from match_results r where r.tournament_id = t.id),
    t.updated_at)
where status = 'finished';

alter table tournaments
    add constraint tournaments_finished_at check ((status = 'finished') = (finished_at is not null));

-- Encerrar marca a hora; reabrir (uma correção que volta a deixar partida por
-- jogar) tira a marca, e o próximo encerramento marca de novo.
create function tournaments_finished_at() returns trigger
language plpgsql as $$
begin
    if new.status = 'finished' then
        if tg_op = 'INSERT' or old.status <> 'finished' then
            new.finished_at := coalesce(new.finished_at, now());
        end if;
    else
        new.finished_at := null;
    end if;
    return new;
end $$;

create trigger tournaments_finished_at
    before insert or update of status on tournaments
    for each row execute function tournaments_finished_at();

-- 2. Os prints. O mapa é a posição na série (fase, partida e número), não a
-- linha de match_games: o jogador anexa antes de enviar o mapa. A linha nasce
-- quando a API entrega a URL de upload (attached_at nulo) e passa a valer
-- quando o upload é conferido no bucket.
create table match_prints (
    id            bigint generated always as identity primary key,
    tenant_id     bigint not null,
    tournament_id bigint not null,
    -- Posição da fase (tournament_stages.position).
    stage         smallint not null,
    -- engine::core::MatchId legível: wb-1-3, lb-2-1, gf-1-1.
    match_key     text not null check (match_key ~ '^(wb|lb|gf)-[1-9][0-9]*-[1-9][0-9]*$'),
    -- Número do mapa na série, a partir de 1.
    number        smallint not null check (number between 1 and 255),
    -- Nome do objeto no bucket, legível no console da AWS:
    -- prints/<comunidade>/<torneio>/<partida>/mapa-<n>-<aleatório>.<ext>, com os
    -- slugs (que não mudam) e 128 bits aleatórios em base58.
    object_key    text not null unique,
    content_type  text not null check (content_type in ('image/webp', 'image/jpeg')),
    -- Bytes; até 2 MB (a mesma constante da API).
    size          integer not null check (size between 1 and 2097152),
    -- Quem anexou; nulo se o usuário foi excluído.
    uploaded_by   bigint,
    created_at    timestamptz not null default now(),
    -- Quando o upload foi conferido no bucket (nulo: pedido e não registrado,
    -- não aparece nem conta no limite).
    attached_at   timestamptz,
    foreign key (tenant_id, tournament_id) references tournaments (tenant_id, id)
        on delete cascade,
    foreign key (tournament_id, stage) references tournament_stages (tournament_id, position),
    foreign key (tenant_id, uploaded_by) references tenant_members (tenant_id, user_id)
        on delete set null (uploaded_by),
    -- O nome tem a partida e o mapa da linha e 128 bits aleatórios em base58;
    -- os slugs da comunidade e do torneio, o trigger match_prints_guard confere.
    constraint match_prints_object_key check (object_key ~ (
        '^prints/[a-z0-9]+(-[a-z0-9]+)*/[a-z0-9]+(-[a-z0-9]+)*/' || match_key
        || '/mapa-' || number || '-[1-9A-HJ-NP-Za-km-z]{22}\.(webp|jpg)$')),
    constraint match_prints_extension
        check ((content_type = 'image/webp') = (object_key like '%.webp'))
);

create index match_prints_game_idx on match_prints (tournament_id, stage, match_key, number);
create index match_prints_pending_idx on match_prints (created_at) where attached_at is null;
create index match_prints_uploaded_by_idx on match_prints (tenant_id, uploaded_by)
    where uploaded_by is not null;

-- Print só existe em torneio iniciado (em andamento ou encerrado) e com até 3
-- por mapa (a constante da API; a API trava o torneio antes de anexar, então
-- a contagem não corre contra outra).
create function match_prints_guard() returns trigger
language plpgsql as $$
begin
    if tg_op = 'INSERT' and not exists (
        select 1 from tournaments
        where id = new.tournament_id and status in ('in_progress', 'finished'))
    then
        raise exception 'print em torneio não iniciado (%)', new.tournament_id
            using errcode = 'check_violation', constraint = 'match_prints_started';
    end if;
    -- O nome começa pelos slugs da comunidade e do torneio da linha (os dois
    -- não mudam; o resto do formato é o check match_prints_object_key).
    if tg_op = 'INSERT' and not exists (
        select 1 from tournaments t join tenants te on te.id = t.tenant_id
        where t.id = new.tournament_id
          and starts_with(new.object_key, 'prints/' || te.slug || '/' || t.slug || '/'))
    then
        raise exception 'print % fora da pasta do torneio %', new.object_key, new.tournament_id
            using errcode = 'check_violation', constraint = 'match_prints_object_key';
    end if;
    if new.attached_at is not null and (
        select count(*) from match_prints
        where tournament_id = new.tournament_id and stage = new.stage
          and match_key = new.match_key and number = new.number
          and attached_at is not null and id <> new.id) >= 3
    then
        raise exception 'mais de 3 prints no mapa % de %', new.number, new.match_key
            using errcode = 'check_violation', constraint = 'match_prints_limit';
    end if;
    if tg_op = 'UPDATE'
       and (new.tenant_id, new.tournament_id, new.stage, new.match_key, new.number,
            new.object_key, new.content_type, new.size)
           is distinct from
           (old.tenant_id, old.tournament_id, old.stage, old.match_key, old.number,
            old.object_key, old.content_type, old.size)
    then
        raise exception 'print % não muda de mapa nem de objeto', old.id
            using errcode = 'check_violation', constraint = 'match_prints_immutable';
    end if;
    return new;
end $$;

create trigger match_prints_guard
    before insert or update on match_prints
    for each row execute function match_prints_guard();

-- 3. A fila do que sai do bucket: o nome de todo print apagado, por qualquer
-- caminho (inclusive a cascata do torneio e do tenant).
create table print_deletions (
    object_key text primary key
               check (object_key ~ ('^prints/[a-z0-9]+(-[a-z0-9]+)*/[a-z0-9]+(-[a-z0-9]+)*/'
                                    || '(wb|lb|gf)-[1-9][0-9]*-[1-9][0-9]*/mapa-[1-9][0-9]*-'
                                    || '[1-9A-HJ-NP-Za-km-z]{22}\.(webp|jpg)$')),
    queued_at  timestamptz not null default now()
);

create function match_prints_deleted() returns trigger
language plpgsql as $$
begin
    insert into print_deletions (object_key) values (old.object_key)
    on conflict (object_key) do nothing;
    return null;
end $$;

create trigger match_prints_deleted
    after delete on match_prints
    for each row execute function match_prints_deleted();

-- 4. O mapa anulado (apagado de match_games: anulado para ser jogado de novo,
-- depois da decisão da série, pelo W.O., por mudar quem joga) leva os prints
-- dele. Corrigir um mapa atualiza a linha e mantém os prints.
create function match_games_prints() returns trigger
language plpgsql as $$
begin
    delete from match_prints
    where tournament_id = old.tournament_id and stage = old.stage
      and match_key = old.match_key and number = old.number;
    return null;
end $$;

create trigger match_games_prints
    after delete on match_games
    for each row execute function match_games_prints();

-- 5. Desfazer o início descarta a chave e, com ela, todos os prints (também os
-- anexados a um mapa ainda não enviado).
create function tournaments_unstarted_prints() returns trigger
language plpgsql as $$
begin
    delete from match_prints where tournament_id = new.id;
    return null;
end $$;

create trigger tournaments_unstarted_prints
    after update of status on tournaments
    for each row
    when (old.status in ('in_progress', 'finished')
          and new.status not in ('in_progress', 'finished'))
    execute function tournaments_unstarted_prints();

-- 6. A última passada da rotina diária dos prints (uma linha só).
create table print_sweeps (
    singleton boolean primary key default true check (singleton),
    swept_at  timestamptz not null
);
