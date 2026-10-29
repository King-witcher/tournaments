-- Limites dos screenshots (29/10):
-- - um anexado por pessoa em cada mapa, no lugar de três por mapa de quem
--   for: com três por mapa, um jogador ocupava as vagas e o adversário ficava
--   sem como anexar a prova dele;
-- - a contagem dos uploads que cada pessoa pede por hora, que remover o
--   screenshot não desfaz (o limite de antes contava só os pendentes, e pedir,
--   mandar e remover repetia sem fim);
-- - até quando vale a URL de upload de cada objeto, levada para a fila de
--   exclusão: a rotina só apaga do bucket depois disso, porque o PUT da mesma
--   URL gravaria de novo um objeto já apagado, sem linha nem fila (o PUT só
--   grava onde não há objeto, If-None-Match na assinatura).

-- 1. A validade da URL de upload. Dados existentes: a URL valia 10 minutos
-- desde a criação da linha; na fila, a hora em que entrou mais 10 minutos (a
-- linha nasceu antes, então a URL já tinha vencido).
alter table match_screenshots add column upload_expires_at timestamptz;
comment on column match_screenshots.upload_expires_at is
    'Até quando a URL de upload do objeto vale.';
update match_screenshots set upload_expires_at = created_at + interval '10 minutes';
alter table match_screenshots
    alter column upload_expires_at set not null,
    add constraint match_screenshots_upload_expires check (upload_expires_at > created_at);

alter table screenshot_deletions add column upload_expires_at timestamptz;
comment on column screenshot_deletions.upload_expires_at is
    'Até quando a URL de upload do objeto vale: antes disso, a rotina não o apaga do bucket.';
update screenshot_deletions set upload_expires_at = queued_at + interval '10 minutes';
alter table screenshot_deletions alter column upload_expires_at set not null;

create or replace function match_screenshots_deleted() returns trigger
language plpgsql as $$
begin
    insert into screenshot_deletions (object_key, upload_expires_at)
    values (old.object_key, old.upload_expires_at)
    on conflict (object_key) do nothing;
    return null;
end $$;

-- 2. O guard, igual ao de 20261118000000_screenshots sem o limite de três por
-- mapa (que vira o índice do passo 3) e com a validade da URL entre o que não
-- muda.
create or replace function match_screenshots_guard() returns trigger
language plpgsql as $$
begin
    if tg_op = 'INSERT' and not exists (
        select 1 from tournaments
        where id = new.tournament_id and status in ('in_progress', 'finished'))
    then
        raise exception 'screenshot em torneio não iniciado (%)', new.tournament_id
            using errcode = 'check_violation', constraint = 'match_screenshots_started';
    end if;
    -- O nome começa pelos slugs da comunidade e do torneio da linha (os dois
    -- não mudam; o resto do formato é o check match_screenshots_object_key).
    if tg_op = 'INSERT' and not exists (
        select 1 from tournaments t join tenants te on te.id = t.tenant_id
        where t.id = new.tournament_id
          and starts_with(new.object_key, 'screenshots/' || te.slug || '/' || t.slug || '/'))
    then
        raise exception 'screenshot % fora da pasta do torneio %', new.object_key, new.tournament_id
            using errcode = 'check_violation', constraint = 'match_screenshots_object_key';
    end if;
    if tg_op = 'UPDATE'
       and (new.tenant_id, new.tournament_id, new.stage, new.match_key, new.number,
            new.object_key, new.content_type, new.size, new.upload_expires_at)
           is distinct from
           (old.tenant_id, old.tournament_id, old.stage, old.match_key, old.number,
            old.object_key, old.content_type, old.size, old.upload_expires_at)
    then
        raise exception 'screenshot % não muda de mapa nem de objeto', old.id
            using errcode = 'check_violation', constraint = 'match_screenshots_immutable';
    end if;
    return new;
end $$;

-- 3. Um anexado por pessoa em cada mapa (pendente não conta; sem autor, a
-- conta que não existe mais, também não). Dados existentes: de quem tem mais
-- de um no mesmo mapa fica o anexado primeiro (no empate, o de menor id); os
-- outros saem, e o trigger põe os objetos na fila de exclusão.
delete from match_screenshots p
using match_screenshots kept
where kept.tournament_id = p.tournament_id and kept.stage = p.stage
  and kept.match_key = p.match_key and kept.number = p.number
  and kept.uploaded_by = p.uploaded_by
  and kept.attached_at is not null and p.attached_at is not null
  and (kept.attached_at, kept.id) < (p.attached_at, p.id);

create unique index match_screenshots_one_per_person
    on match_screenshots (tournament_id, stage, match_key, number, uploaded_by)
    where attached_at is not null;

-- 4. Uploads pedidos por pessoa em cada hora, em todas as comunidades. Cada
-- URL entregue soma um, e nada desconta: nem registrar, nem remover, nem a
-- rotina apagar o upload abandonado. A rotina diária apaga as horas passadas.
create table screenshot_upload_counts (
    -- Contagem da própria pessoa, que só existe com ela.
    user_id bigint not null references users (id) on delete cascade,
    -- O começo da hora, em UTC.
    hour    timestamptz not null check (hour = date_trunc('hour', hour, 'UTC')),
    uploads integer not null check (uploads > 0),
    primary key (user_id, hour)
);
