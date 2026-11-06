-- Foto de perfil enviada (06/11): no bucket público, como o logo e a capa
-- (images.md). O banco guarda só o nome do objeto (avatars/<aleatório>.<ext>,
-- sem pasta: a pessoa não tem slug, e o id interno não vai para a URL). A
-- foto vale mais que o avatar do Discord (users.avatar_url), que continua
-- gravado e volta quando a foto sai.

-- 1. A foto: um objeto é de uma conta só; a conta excluída não tem.
alter table users
    add column avatar_key text unique
        constraint users_avatar_key
        check (avatar_key ~ '^avatars/[1-9A-HJ-NP-Za-km-z]{22}\.(webp|jpg)$'),
    add constraint users_deleted_avatar check (deleted_at is null or avatar_key is null);
comment on column users.avatar_key is
    'Nome do objeto da foto enviada no bucket público (avatars/<aleatório>.<ext>); nulo: sem foto (vale o avatar do Discord, se houver).';

-- 2. O upload pendente da foto: sem comunidade, com a conta dona. Os da
-- comunidade continuam com tenant_id (e as FKs compostas dela, que não valem
-- com tenant_id nulo).
alter table image_uploads
    alter column tenant_id drop not null,
    add column user_id bigint references users (id) on delete cascade,
    add constraint image_uploads_scope check (
        (tenant_id is null) = (user_id is not null)
        and (user_id is null
             or (tournament_id is null and uploaded_by is not null and uploaded_by = user_id))
    ),
    drop constraint image_uploads_object_key,
    add constraint image_uploads_object_key check (object_key ~ (
        case
            when user_id is not null then '^avatars/'
            when tournament_id is null then '^logos/[a-z0-9]+(-[a-z0-9]+)*/'
            else '^covers/[a-z0-9]+(-[a-z0-9]+)*/[a-z0-9]+(-[a-z0-9]+)*/'
        end || '[1-9A-HJ-NP-Za-km-z]{22}\.(webp|jpg)$'));
comment on column image_uploads.user_id is
    'A foto de perfil desta conta (com tenant_id nulo); nulo: o logo ou a capa da comunidade.';

create index image_uploads_user_idx on image_uploads (user_id) where user_id is not null;

-- 3. A fila aceita as fotos.
alter table image_deletions
    drop constraint image_deletions_object_key_check,
    add constraint image_deletions_object_key_check check (object_key ~ (
        '^(avatars|logos/[a-z0-9]+(-[a-z0-9]+)*|covers/[a-z0-9]+(-[a-z0-9]+)*/[a-z0-9]+(-[a-z0-9]+)*)/'
        || '[1-9A-HJ-NP-Za-km-z]{22}\.(webp|jpg)$'));

-- 4. Foto trocada, removida ou da conta excluída vai para a fila, como o logo
-- e a capa; o upload que sai sem virar foto também.
create function users_avatar_released() returns trigger
language plpgsql as $$
begin
    if old.avatar_key is not null
       and (tg_op = 'DELETE' or new.avatar_key is distinct from old.avatar_key)
    then
        perform image_released(old.avatar_key, now() + interval '10 minutes');
    end if;
    return null;
end $$;

create trigger users_avatar_released
    after update of avatar_key or delete on users
    for each row execute function users_avatar_released();

create or replace function image_uploads_deleted() returns trigger
language plpgsql as $$
begin
    if not exists (select 1 from tenants where logo_key = old.object_key)
       and not exists (select 1 from tournaments where cover_key = old.object_key)
       and not exists (select 1 from users where avatar_key = old.object_key)
    then
        perform image_released(old.object_key, old.upload_expires_at);
    end if;
    return null;
end $$;
