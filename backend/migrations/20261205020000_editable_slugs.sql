-- Endereço editável e torneio excluível (03/11, docs/decisions/tenants.md e
-- tournaments.md): o owner troca o slug da comunidade, e a organização, o do
-- torneio enquanto ele é rascunho. O endereço antigo fica livre na hora, sem
-- redirecionamento nem histórico. E o torneio publicado também pode ser
-- excluído (quem pode é regra da API; o banco já aceitava a exclusão).
--
-- Dados existentes: nada a corrigir. Os checks novos só afrouxam os de antes,
-- e todo nome de objeto que passava passa.

-- 1. O slug da comunidade muda; o do torneio, só no rascunho (depois de
-- publicado, há links e avisos com ele). Tenant e idioma original continuam
-- fixos, e o resto do guard é o de 20261030010000_consistency_checks.
drop trigger tenants_slug_immutable on tenants;
drop function tenants_slug_immutable();

create or replace function tournaments_guard() returns trigger
language plpgsql as $$
begin
    if (new.tenant_id, new.locale) is distinct from (old.tenant_id, old.locale)
    then
        raise exception 'tenant e idioma original do torneio % não mudam', old.id
            using errcode = 'check_violation', constraint = 'tournaments_identity_immutable';
    end if;
    if new.slug <> old.slug and old.status <> 'draft' then
        raise exception 'o slug do torneio % só muda no rascunho', old.id
            using errcode = 'check_violation', constraint = 'tournaments_slug_after_draft';
    end if;
    if new.max_participants < old.max_participants
       and new.max_participants < (select count(*) from participants where tournament_id = new.id)
    then
        raise exception 'torneio % com menos vagas que inscritos', old.id
            using errcode = 'check_violation', constraint = 'tournaments_max_participants_below_registered';
    end if;
    return new;
end $$;

-- 2. Objetos no bucket não se movem: o nome guarda os slugs de quando foi
-- criado e vale para sempre. A forma continua conferida, mas não mais contra o
-- slug atual (o de antes recusaria a própria troca). O nome continua único: a
-- fila de exclusão apaga pelo nome exato, e os 128 bits aleatórios o separam
-- de um objeto da comunidade que pegar o endereço antigo.
alter table tenants drop constraint tenants_logo_key;
alter table tenants add constraint tenants_logo_key
    check (logo_key ~ '^logos/[a-z0-9]+(-[a-z0-9]+)*/[1-9A-HJ-NP-Za-km-z]{22}\.(webp|jpg)$');
comment on column tenants.logo_key is
    'Nome do objeto do logo no bucket público (logos/<comunidade quando foi enviado>/<aleatório>.<ext>); nulo: sem logo.';

alter table tournaments drop constraint tournaments_cover_key;
alter table tournaments add constraint tournaments_cover_key
    check (cover_key ~ ('^covers/[a-z0-9]+(-[a-z0-9]+)*/[a-z0-9]+(-[a-z0-9]+)*/'
                        || '[1-9A-HJ-NP-Za-km-z]{22}\.(webp|jpg)$'));
comment on column tournaments.cover_key is
    'Nome do objeto da capa no bucket público (covers/<comunidade>/<torneio>, de quando foi enviada, /<aleatório>.<ext>); nulo: sem capa.';

-- O guard dos screenshots, igual ao de 20261120000000_screenshot_limits sem a
-- conferência da pasta contra os slugs: o que a API leu antes de uma troca
-- ainda vale. O formato é o check match_screenshots_object_key.
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

-- 3. O endereço também está escrito no sininho, na fila do Discord e nos
-- avisos ainda no outbox (/{comunidade}/{torneio}/...). Os dois gatilhos abaixo
-- os acompanham na mesma transação, por qualquer caminho: quem fica de fora é o
-- que já saiu (a mensagem postada no Discord, o Web Push entregue).
--
-- Os outros eventos do outbox (tenant, tournament, access_lost) só dizem "isto
-- mudou" a quem assina aquele slug: com o slug velho, vão às abas que ainda
-- estão nele, que é o certo, e não ficam guardados.
--
-- `root` é o começo do caminho, sem a barra do fim ('/quake-br').
create function path_under(path text, root text) returns boolean
language sql immutable as $$
    select path = root or starts_with(path, root || '/')
$$;

create function path_moved(path text, root text, moved_to text) returns text
language sql immutable as $$
    select case when path_under(path, root)
                then moved_to || substr(path, length(root) + 1)
                else path end
$$;

-- A comunidade mudou de endereço. Só as notificações de quem é ou foi membro
-- (a pessoa que saiu continua com a linha em tenant_members): é por essa pessoa
-- que o índice encontra, e o caminho de outra comunidade não muda. Assim o
-- sininho não leva à comunidade que pegar o endereço antigo. O torneio não
-- precisa disto: só muda de endereço no rascunho, que nunca avisou ninguém.
create function tenants_slug_moved() returns trigger
language plpgsql as $$
declare
    root text := '/' || old.slug;
    moved_to text := '/' || new.slug;
begin
    update notifications
    set notice = jsonb_set(notice, '{path}', to_jsonb(path_moved(notice ->> 'path', root, moved_to)))
    where user_id in (select user_id from tenant_members where tenant_id = new.id)
      and path_under(notice ->> 'path', root);
    -- merge_key é "{código}:{caminho}" (discord/queue.rs): avisos iguais se
    -- juntam com os que entrarem depois, já pelo caminho novo.
    update discord_messages
    set path = path_moved(path, root, moved_to),
        merge_key = case when merge_key is not null
                         then code || ':' || path_moved(path, root, moved_to) end
    where tenant_id = new.id and path_under(path, root);
    update outbox
    set event = jsonb_set(
            event,
            array[event ->> 'kind', 'path'],
            to_jsonb(path_moved(event #>> array[event ->> 'kind', 'path'], root, moved_to)))
    where event ->> 'kind' in ('notice', 'announce')
      and path_under(event #>> array[event ->> 'kind', 'path'], root);
    return null;
end $$;

create trigger tenants_slug_moved
    after update of slug on tenants
    for each row when (old.slug is distinct from new.slug)
    execute function tenants_slug_moved();

-- O torneio foi excluído: nada mais leva a ele nem avisa dele. Na cascata do
-- tenant ele já não está: a fila do Discord vai pela cascata dela e o caminho
-- não se calcula (`root` nulo).
create function tournaments_path_forgotten() returns trigger
language plpgsql as $$
declare
    root text := (select '/' || slug || '/' || old.slug from tenants where id = old.tenant_id);
begin
    if root is null then
        return null;
    end if;
    delete from notifications
    where user_id in (select user_id from tenant_members where tenant_id = old.tenant_id)
      and path_under(notice ->> 'path', root);
    delete from discord_messages
    where tenant_id = old.tenant_id and path_under(path, root);
    delete from outbox
    where event ->> 'kind' in ('notice', 'announce')
      and path_under(event #>> array[event ->> 'kind', 'path'], root);
    return null;
end $$;

create trigger tournaments_path_forgotten
    after delete on tournaments
    for each row execute function tournaments_path_forgotten();

-- 4. O evento que leva as abas abertas ao endereço novo. O check do outbox só
-- ganha um tipo (todo evento que passava passa).
create or replace function outbox_event_valid(event jsonb) returns boolean
language sql immutable as $$
    select coalesce(
        jsonb_typeof(event) = 'object'
        and event ->> 'kind' in
            ('tenant', 'tournament', 'me', 'access_lost', 'sessions_ended', 'notice',
             'announce', 'unread', 'moved'),
        false)
$$;
