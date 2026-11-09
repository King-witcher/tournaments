-- Capa da comunidade (09/11, images.md): opcional, 2:1, ao lado do logo. Vale
-- na prévia do link de convite e no lugar da capa que falta num torneio. O
-- objeto fica na pasta das capas, sem a do torneio
-- (covers/<comunidade>/<aleatório>.<ext>).

-- 1. A capa: como o logo, a pasta é o endereço de quando foi enviada.
alter table tenants
    add column cover_key text unique
        constraint tenants_cover_key
        check (cover_key ~ '^covers/[a-z0-9]+(-[a-z0-9]+)*/[1-9A-HJ-NP-Za-km-z]{22}\.(webp|jpg)$');
comment on column tenants.cover_key is
    'Nome do objeto da capa da comunidade no bucket público (covers/<comunidade quando foi enviada>/<aleatório>.<ext>); nulo: sem capa.';

-- 2. O upload pendente diz de quem é a capa. As linhas que existem (de no
-- máximo um dia) são todas de torneio: a comunidade não tinha capa.
alter table image_uploads
    drop constraint image_uploads_kind,
    drop constraint image_uploads_scope,
    drop constraint image_uploads_object_key;
update image_uploads set kind = 'tournament_cover' where kind = 'cover';
alter table image_uploads
    add constraint image_uploads_kind
        check (kind in ('logo', 'tenant_cover', 'tournament_cover', 'avatar', 'rules')),
    add constraint image_uploads_scope check (
        (tenant_id is null) = (kind = 'avatar')
        and (user_id is not null) = (kind = 'avatar')
        and (tournament_id is null) = (kind in ('logo', 'tenant_cover', 'avatar'))
        and (kind <> 'avatar' or uploaded_by is not distinct from user_id)
    ),
    add constraint image_uploads_object_key check (object_key ~ (
        case kind
            when 'avatar' then '^avatars/'
            when 'logo' then '^logos/[a-z0-9]+(-[a-z0-9]+)*/'
            when 'tenant_cover' then '^covers/[a-z0-9]+(-[a-z0-9]+)*/'
            when 'tournament_cover' then '^covers/[a-z0-9]+(-[a-z0-9]+)*/[a-z0-9]+(-[a-z0-9]+)*/'
            else '^rules/[a-z0-9]+(-[a-z0-9]+)*/'
        end || '[1-9A-HJ-NP-Za-km-z]{22}\.(webp|jpg)$'));
comment on column image_uploads.kind is
    'O que o upload vira: logo, tenant_cover (a capa da comunidade), tournament_cover (a capa de tournament_id), avatar (a foto de user_id) ou rules (imagem das regras, pedida em tournament_id e registrada na comunidade).';

-- 3. A fila aceita as capas de comunidade.
alter table image_deletions
    drop constraint image_deletions_object_key_check,
    add constraint image_deletions_object_key_check check (object_key ~ (
        '^(avatars|logos/[a-z0-9]+(-[a-z0-9]+)*|covers/[a-z0-9]+(-[a-z0-9]+)*(/[a-z0-9]+(-[a-z0-9]+)*)?'
        || '|rules/[a-z0-9]+(-[a-z0-9]+)*)/'
        || '[1-9A-HJ-NP-Za-km-z]{22}\.(webp|jpg)$'));

-- 4. Capa trocada, removida ou da comunidade apagada vai para a fila, como o
-- logo; o upload que vira capa, não.
create function tenants_cover_released() returns trigger
language plpgsql as $$
begin
    if old.cover_key is not null
       and (tg_op = 'DELETE' or new.cover_key is distinct from old.cover_key)
    then
        perform image_released(old.cover_key, now() + interval '10 minutes');
    end if;
    return null;
end $$;

create trigger tenants_cover_released
    after update of cover_key or delete on tenants
    for each row execute function tenants_cover_released();

create or replace function image_uploads_deleted() returns trigger
language plpgsql as $$
begin
    if not exists (select 1 from tenants
                   where logo_key = old.object_key or cover_key = old.object_key)
       and not exists (select 1 from tournaments where cover_key = old.object_key)
       and not exists (select 1 from users where avatar_key = old.object_key)
       and not exists (select 1 from rules_images where object_key = old.object_key)
    then
        perform image_released(old.object_key, old.upload_expires_at);
    end if;
    return null;
end $$;
