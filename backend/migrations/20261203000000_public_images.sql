-- Imagens públicas (01/11): o logo da comunidade e a capa do torneio, no
-- bucket público, que o CDN serve. O banco guarda só o nome do objeto; a API
-- monta a URL com o endereço do CDN (S3_PUBLIC_URL). Como nos screenshots, o
-- navegador manda o arquivo direto ao bucket por uma URL assinada: o upload
-- pedido fica em image_uploads até o registro, e todo objeto que deixa de
-- ser usado (trocado, removido, abandonado, da comunidade ou do torneio
-- apagados) entra na fila image_deletions, que a rotina diária esvazia.

-- 1. O logo e a capa: o nome do objeto, na pasta dos slugs (que não mudam),
-- com 128 bits aleatórios em base58, como os screenshots. Um objeto é de uma
-- linha só.
alter table tenants
    add column logo_key text unique
        constraint tenants_logo_key
        check (logo_key ~ ('^logos/' || slug || '/[1-9A-HJ-NP-Za-km-z]{22}\.(webp|jpg)$'));
comment on column tenants.logo_key is
    'Nome do objeto do logo no bucket público (logos/<comunidade>/<aleatório>.<ext>); nulo: sem logo.';

alter table tournaments
    add column cover_key text unique
        constraint tournaments_cover_key
        check (cover_key ~ ('^covers/[a-z0-9]+(-[a-z0-9]+)*/' || slug
                            || '/[1-9A-HJ-NP-Za-km-z]{22}\.(webp|jpg)$'));
comment on column tournaments.cover_key is
    'Nome do objeto da capa no bucket público (covers/<comunidade>/<torneio>/<aleatório>.<ext>); nulo: sem capa.';

-- 2. Uploads pedidos e ainda não registrados: o registro tira a linha (e o
-- objeto passa a ser o logo ou a capa); a rotina diária apaga os
-- abandonados.
create table image_uploads (
    id                bigint generated always as identity primary key,
    tenant_id         bigint not null references tenants (id) on delete cascade,
    -- A capa deste torneio; nulo: o logo da comunidade.
    tournament_id     bigint,
    object_key        text not null unique,
    content_type      text not null check (content_type in ('image/webp', 'image/jpeg')),
    -- Bytes, até o MAX_BYTES da API.
    size              integer not null check (size between 1 and 1048576),
    -- Quem pediu, o único que registra. Nulo: a pessoa saiu da comunidade ou
    -- a conta foi apagada.
    uploaded_by       bigint,
    -- Até quando a URL de upload vale.
    upload_expires_at timestamptz not null,
    created_at        timestamptz not null default now(),
    foreign key (tenant_id, tournament_id) references tournaments (tenant_id, id)
        on delete cascade,
    foreign key (tenant_id, uploaded_by) references tenant_members (tenant_id, user_id)
        on delete set null (uploaded_by),
    constraint image_uploads_upload_expires check (upload_expires_at > created_at),
    constraint image_uploads_object_key check (object_key ~ (
        '^' || case when tournament_id is null then 'logos' else 'covers/[a-z0-9]+(-[a-z0-9]+)*' end
        || '/[a-z0-9]+(-[a-z0-9]+)*/[1-9A-HJ-NP-Za-km-z]{22}\.(webp|jpg)$'))
);

create index image_uploads_uploaded_by_idx on image_uploads (uploaded_by, created_at)
    where uploaded_by is not null;
create index image_uploads_tournament_idx on image_uploads (tenant_id, tournament_id)
    where tournament_id is not null;

-- 3. A fila do que sai do bucket público e a última passada da rotina.
create table image_deletions (
    object_key        text primary key
                      check (object_key ~ ('^(logos|covers/[a-z0-9]+(-[a-z0-9]+)*)/'
                                           || '[a-z0-9]+(-[a-z0-9]+)*/'
                                           || '[1-9A-HJ-NP-Za-km-z]{22}\.(webp|jpg)$')),
    -- Até quando uma URL de upload do objeto ainda pode valer: antes disso, a
    -- rotina não o apaga (o mesmo PUT gravaria de novo um objeto já apagado).
    upload_expires_at timestamptz not null,
    queued_at         timestamptz not null default now()
);

create table image_sweeps (
    singleton boolean primary key default true check (singleton),
    swept_at  timestamptz not null
);

-- 4. Todo objeto que deixa de ser usado entra na fila, por qualquer caminho
-- (inclusive a cascata do tenant e do torneio). Logo ou capa trocados,
-- removidos ou apagados: a URL de upload deles valeu 10 minutos
-- (UPLOAD_EXPIRES da API) a partir de um momento anterior a agora.
create function image_released(key text, expires timestamptz) returns void
language sql as $$
    insert into image_deletions (object_key, upload_expires_at)
    values (key, expires)
    on conflict (object_key) do nothing;
$$;

create function tenants_logo_released() returns trigger
language plpgsql as $$
begin
    if old.logo_key is not null
       and (tg_op = 'DELETE' or new.logo_key is distinct from old.logo_key)
    then
        perform image_released(old.logo_key, now() + interval '10 minutes');
    end if;
    return null;
end $$;

create trigger tenants_logo_released
    after update of logo_key or delete on tenants
    for each row execute function tenants_logo_released();

create function tournaments_cover_released() returns trigger
language plpgsql as $$
begin
    if old.cover_key is not null
       and (tg_op = 'DELETE' or new.cover_key is distinct from old.cover_key)
    then
        perform image_released(old.cover_key, now() + interval '10 minutes');
    end if;
    return null;
end $$;

create trigger tournaments_cover_released
    after update of cover_key or delete on tournaments
    for each row execute function tournaments_cover_released();

-- O upload que sai sem virar logo nem capa (abandonado, recusado, levado pela
-- cascata) leva o objeto para a fila. O registro grava o logo ou a capa antes
-- de apagar a linha, e aí o objeto fica.
create function image_uploads_deleted() returns trigger
language plpgsql as $$
begin
    if not exists (select 1 from tenants where logo_key = old.object_key)
       and not exists (select 1 from tournaments where cover_key = old.object_key)
    then
        perform image_released(old.object_key, old.upload_expires_at);
    end if;
    return null;
end $$;

create trigger image_uploads_deleted
    after delete on image_uploads
    for each row execute function image_uploads_deleted();
