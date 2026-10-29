-- "Print" passa a "screenshot", o nome no código, na API e no bucket (na
-- interface, "captura de tela"): as tabelas, a coluna do torneio,
-- constraints, índices, sequência, triggers e o prefixo dos objetos no bucket
-- (prints/ vira screenshots/). As regras são as de 20261112000000_prints.sql,
-- sem mudança.

-- 1. Os triggers e as funções saem antes de renomear: o corpo de uma função
-- não acompanha o nome novo da tabela, e o guard recusaria a troca de nome dos
-- objetos (passo 4). Voltam com os nomes novos no passo 6.
drop trigger match_prints_guard on match_prints;
drop trigger match_prints_deleted on match_prints;
drop trigger match_games_prints on match_games;
drop trigger tournaments_unstarted_prints on tournaments;
drop function match_prints_guard();
drop function match_prints_deleted();
drop function match_games_prints();
drop function tournaments_unstarted_prints();

-- Os checks do nome do objeto voltam com o prefixo novo no passo 5.
alter table match_prints drop constraint match_prints_object_key;
alter table print_deletions drop constraint print_deletions_object_key_check;

-- 2. Tabelas e coluna.
alter table tournaments rename column prints_required to screenshots_required;
alter table match_prints rename to match_screenshots;
alter table print_deletions rename to screenshot_deletions;
alter table print_sweeps rename to screenshot_sweeps;

-- 3. O resto dos nomes: constraints (os índices delas mudam junto), os índices
-- criados à mão e a sequência do id. Pelo catálogo, porque os nomes gerados
-- variam com a versão do Postgres (o 18 também nomeia os not null).
do $$
declare
    r record;
begin
    for r in
        select conrelid::regclass as tbl, conname
        from pg_constraint
        where connamespace = 'public'::regnamespace
          and conname ~ '^(match_prints|print_deletions|print_sweeps|tournaments_prints_required)_'
    loop
        execute format('alter table %s rename constraint %I to %I',
                       r.tbl, r.conname, replace(r.conname, 'print', 'screenshot'));
    end loop;
    for r in
        select oid::regclass as rel, relname, relkind
        from pg_class
        where relnamespace = 'public'::regnamespace and relkind in ('i', 'S')
          and relname ~ '^(match_prints|print_deletions|print_sweeps)_'
    loop
        execute format('alter %s %s rename to %I',
                       case r.relkind when 'S' then 'sequence' else 'index' end,
                       r.rel, replace(r.relname, 'print', 'screenshot'));
    end loop;
end $$;

-- 4. Dados existentes: o nome gravado de cada objeto troca o prefixo. Os
-- objetos que já estiverem num bucket mudam junto, por fora:
-- aws s3 mv s3://<bucket>/prints/ s3://<bucket>/screenshots/ --recursive
update match_screenshots
set object_key = 'screenshots/' || substr(object_key, length('prints/') + 1)
where starts_with(object_key, 'prints/');

update screenshot_deletions
set object_key = 'screenshots/' || substr(object_key, length('prints/') + 1)
where starts_with(object_key, 'prints/');

-- 5. O formato do nome, com o prefixo novo.
alter table match_screenshots
    add constraint match_screenshots_object_key check (object_key ~ (
        '^screenshots/[a-z0-9]+(-[a-z0-9]+)*/[a-z0-9]+(-[a-z0-9]+)*/' || match_key
        || '/mapa-' || number || '-[1-9A-HJ-NP-Za-km-z]{22}\.(webp|jpg)$'));

alter table screenshot_deletions
    add constraint screenshot_deletions_object_key_check
        check (object_key ~ ('^screenshots/[a-z0-9]+(-[a-z0-9]+)*/[a-z0-9]+(-[a-z0-9]+)*/'
                             || '(wb|lb|gf)-[1-9][0-9]*-[1-9][0-9]*/mapa-[1-9][0-9]*-'
                             || '[1-9A-HJ-NP-Za-km-z]{22}\.(webp|jpg)$'));

-- 6. Os triggers, iguais aos de antes com os nomes novos.

-- Screenshot só existe em torneio iniciado, na pasta do torneio, até 3 por
-- mapa, e não muda de mapa nem de objeto.
create function match_screenshots_guard() returns trigger
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
    if new.attached_at is not null and (
        select count(*) from match_screenshots
        where tournament_id = new.tournament_id and stage = new.stage
          and match_key = new.match_key and number = new.number
          and attached_at is not null and id <> new.id) >= 3
    then
        raise exception 'mais de 3 screenshots no mapa % de %', new.number, new.match_key
            using errcode = 'check_violation', constraint = 'match_screenshots_limit';
    end if;
    if tg_op = 'UPDATE'
       and (new.tenant_id, new.tournament_id, new.stage, new.match_key, new.number,
            new.object_key, new.content_type, new.size)
           is distinct from
           (old.tenant_id, old.tournament_id, old.stage, old.match_key, old.number,
            old.object_key, old.content_type, old.size)
    then
        raise exception 'screenshot % não muda de mapa nem de objeto', old.id
            using errcode = 'check_violation', constraint = 'match_screenshots_immutable';
    end if;
    return new;
end $$;

create trigger match_screenshots_guard
    before insert or update on match_screenshots
    for each row execute function match_screenshots_guard();

-- Todo screenshot apagado, por qualquer caminho (inclusive a cascata do
-- torneio e do tenant), deixa o nome na fila do que sai do bucket.
create function match_screenshots_deleted() returns trigger
language plpgsql as $$
begin
    insert into screenshot_deletions (object_key) values (old.object_key)
    on conflict (object_key) do nothing;
    return null;
end $$;

create trigger match_screenshots_deleted
    after delete on match_screenshots
    for each row execute function match_screenshots_deleted();

-- O mapa anulado (apagado de match_games) leva os screenshots dele.
create function match_games_screenshots() returns trigger
language plpgsql as $$
begin
    delete from match_screenshots
    where tournament_id = old.tournament_id and stage = old.stage
      and match_key = old.match_key and number = old.number;
    return null;
end $$;

create trigger match_games_screenshots
    after delete on match_games
    for each row execute function match_games_screenshots();

-- Desfazer o início leva todos os screenshots do torneio.
create function tournaments_unstarted_screenshots() returns trigger
language plpgsql as $$
begin
    delete from match_screenshots where tournament_id = new.id;
    return null;
end $$;

create trigger tournaments_unstarted_screenshots
    after update of status on tournaments
    for each row
    when (old.status in ('in_progress', 'finished')
          and new.status not in ('in_progress', 'finished'))
    execute function tournaments_unstarted_screenshots();
