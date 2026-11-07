-- Imagens nas regras da organização (07/11, images.md): só pelo nosso upload,
-- no bucket público, como o logo e a capa. A imagem é da comunidade
-- (rules/<comunidade>/<aleatório>.<ext>), não do torneio: copiar um torneio
-- copia o texto das regras, com as URLs, e a imagem vale nos dois. O banco
-- sabe quais textos (o original e cada tradução) usam cada imagem; a que fica
-- sem texto nenhum sai na hora para a fila image_deletions, e a enviada e
-- nunca usada, pela rotina diária depois de um dia.

-- 1. O upload pendente diz o que vai ser. Antes, o tipo saía de quais colunas
-- estavam preenchidas, e a imagem das regras, enviada num torneio, ficaria
-- igual à capa. As linhas que existem (pendentes, de no máximo um dia) ganham
-- o tipo pela regra de antes: com conta, a foto; sem torneio, o logo; com
-- torneio, a capa.
alter table image_uploads add column kind text;
update image_uploads
set kind = case
    when user_id is not null then 'avatar'
    when tournament_id is null then 'logo'
    else 'cover'
end;
alter table image_uploads
    alter column kind set not null,
    add constraint image_uploads_kind check (kind in ('logo', 'cover', 'avatar', 'rules')),
    drop constraint image_uploads_scope,
    add constraint image_uploads_scope check (
        (tenant_id is null) = (kind = 'avatar')
        and (user_id is not null) = (kind = 'avatar')
        and (tournament_id is null) = (kind in ('logo', 'avatar'))
        and (kind <> 'avatar' or uploaded_by is not distinct from user_id)
    ),
    drop constraint image_uploads_object_key,
    add constraint image_uploads_object_key check (object_key ~ (
        case kind
            when 'avatar' then '^avatars/'
            when 'logo' then '^logos/[a-z0-9]+(-[a-z0-9]+)*/'
            when 'cover' then '^covers/[a-z0-9]+(-[a-z0-9]+)*/[a-z0-9]+(-[a-z0-9]+)*/'
            else '^rules/[a-z0-9]+(-[a-z0-9]+)*/'
        end || '[1-9A-HJ-NP-Za-km-z]{22}\.(webp|jpg)$'));
comment on column image_uploads.kind is
    'O que o upload vira: logo, cover (a capa de tournament_id), avatar (a foto de user_id) ou rules (imagem das regras, pedida em tournament_id e registrada na comunidade).';

-- 2. As imagens das regras registradas na comunidade. O nome do objeto tem a
-- pasta do endereço de quando foi enviado, como o logo: o banco confere a
-- forma, e a comunidade dona é a coluna, não a pasta.
create table rules_images (
    id         bigint generated always as identity primary key,
    tenant_id  bigint not null references tenants (id) on delete cascade,
    object_key text not null unique
               constraint rules_images_object_key
               check (object_key ~ '^rules/[a-z0-9]+(-[a-z0-9]+)*/[1-9A-HJ-NP-Za-km-z]{22}\.(webp|jpg)$'),
    created_at timestamptz not null default now(),
    unique (tenant_id, id)
);

-- Quais textos usam cada imagem: o original do torneio (locale nulo) ou a
-- tradução nesse idioma. A API regrava a lista a cada texto salvo; o
-- torneio e a tradução apagados levam as linhas deles.
create table rules_image_refs (
    tenant_id     bigint not null,
    tournament_id bigint not null,
    -- A tradução neste idioma; nulo: o texto original.
    locale        text,
    image_id      bigint not null,
    foreign key (tenant_id, tournament_id) references tournaments (tenant_id, id)
        on delete cascade,
    foreign key (tournament_id, locale) references tournament_translations (tournament_id, locale)
        on delete cascade,
    -- Imagem em uso não sai (no action: a cascata da comunidade passa).
    foreign key (tenant_id, image_id) references rules_images (tenant_id, id)
);

create unique index rules_image_refs_original on rules_image_refs (tournament_id, image_id)
    where locale is null;
create unique index rules_image_refs_translation
    on rules_image_refs (tournament_id, locale, image_id)
    where locale is not null;
create index rules_image_refs_image_idx on rules_image_refs (image_id);
create index rules_images_unused_idx on rules_images (tenant_id, created_at);

-- 3. A fila aceita as imagens das regras.
alter table image_deletions
    drop constraint image_deletions_object_key_check,
    add constraint image_deletions_object_key_check check (object_key ~ (
        '^(avatars|logos/[a-z0-9]+(-[a-z0-9]+)*|covers/[a-z0-9]+(-[a-z0-9]+)*/[a-z0-9]+(-[a-z0-9]+)*'
        || '|rules/[a-z0-9]+(-[a-z0-9]+)*)/'
        || '[1-9A-HJ-NP-Za-km-z]{22}\.(webp|jpg)$'));

-- 4. A imagem que sai (sem texto, nunca usada, da comunidade apagada) vai
-- para a fila, como o logo trocado.
create function rules_images_released() returns trigger
language plpgsql as $$
begin
    perform image_released(old.object_key, now() + interval '10 minutes');
    return null;
end $$;

create trigger rules_images_released
    after delete on rules_images
    for each row execute function rules_images_released();

-- O último texto que deixa de usar a imagem (editado, tradução removida,
-- torneio apagado) a leva junto. A trava vem antes da contagem: quem salva um
-- texto com a imagem a trava (for share) até gravar a referência, então
-- nenhum dos dois apaga a imagem que o outro acabou de usar.
create function rules_image_refs_deleted() returns trigger
language plpgsql as $$
begin
    perform 1 from rules_images where id = old.image_id for update;
    if found and not exists (select 1 from rules_image_refs where image_id = old.image_id) then
        delete from rules_images where id = old.image_id;
    end if;
    return null;
end $$;

create trigger rules_image_refs_deleted
    after delete on rules_image_refs
    for each row execute function rules_image_refs_deleted();

-- O upload que vira imagem das regras não vai para a fila.
create or replace function image_uploads_deleted() returns trigger
language plpgsql as $$
begin
    if not exists (select 1 from tenants where logo_key = old.object_key)
       and not exists (select 1 from tournaments where cover_key = old.object_key)
       and not exists (select 1 from users where avatar_key = old.object_key)
       and not exists (select 1 from rules_images where object_key = old.object_key)
    then
        perform image_released(old.object_key, old.upload_expires_at);
    end if;
    return null;
end $$;
