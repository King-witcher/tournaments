-- Coerência e forma dos dados (decisões em "Consistência do banco" no
-- docs/decisions.md): o que a API já garantia passa a ser regra do banco
-- também. Dado antigo que violaria uma regra é corrigido antes dela, de forma
-- determinística. Regra de estado sai com o código de check (23514) e o nome
-- da regra em `constraint`, também quando é trigger.

-- 1. Torneios.

-- Data de início obrigatória fora do rascunho. Torneios publicados antes de
-- existir a data ficaram sem: recebem a data de criação (já passada) e, se
-- ainda estão na inscrição, a abertura automática do check-in fica suspensa
-- como numa reabertura (registration_reopened_at = agora), para o status não
-- andar sozinho; mudar a data ou a janela reativa.
update tournaments
set starts_at = created_at,
    registration_reopened_at = case
        when status = 'registration' then greatest(registration_reopened_at, now())
        else registration_reopened_at
    end
where status <> 'draft' and starts_at is null;

-- Sem janela não existe o status check-in (desligar o check-in durante ele
-- volta para a inscrição).
update tournaments set status = 'registration'
where status = 'check_in' and check_in_minutes is null;

-- Forma mínima do JSON gravado: engine::champions::ChampionRules e
-- engine::formats::StageFormat, com as chaves obrigatórias e os valores que o
-- engine e a API aceitam. Mudar o serde deles exige migration dos dados e
-- destas funções (create or replace não reconfere as linhas gravadas).

-- Melhor de N: inteiro ímpar de 1 a 255 (u8).
create function best_of_valid(n jsonb) returns boolean
language sql immutable as $$
    select case
        when jsonb_typeof(n) = 'number' and n #>> '{}' ~ '^[0-9]{1,3}$'
            then (n #>> '{}')::integer between 1 and 255 and (n #>> '{}')::integer % 2 = 1
        else false
    end
$$;

-- {"bans_per_game": 0..3, "no_repeat": null | {"who", "scope"}}
create function champion_rules_valid(rules jsonb) returns boolean
language sql immutable as $$
    select coalesce(
        jsonb_typeof(rules) = 'object'
        and jsonb_typeof(rules -> 'bans_per_game') = 'number'
        and rules ->> 'bans_per_game' in ('0', '1', '2', '3')
        and case jsonb_typeof(rules -> 'no_repeat')
            when 'null' then true
            when 'object' then rules #>> '{no_repeat,who}' in ('player', 'team')
                and rules #>> '{no_repeat,scope}' in ('match', 'stage', 'tournament')
            else false
        end,
        false)
$$;

-- {"format": "elimination", "kind": {"kind": "single"} | {"kind": "double",
-- "grand_final_reset": bool}, "series": {"default": N, "finals": N}}
create function stage_config_valid(config jsonb) returns boolean
language sql immutable as $$
    select coalesce(
        jsonb_typeof(config) = 'object'
        and config ->> 'format' = 'elimination'
        and jsonb_typeof(config -> 'kind') = 'object'
        and case config #>> '{kind,kind}'
            when 'single' then true
            when 'double' then jsonb_typeof(config #> '{kind,grand_final_reset}') = 'boolean'
            else false
        end
        and jsonb_typeof(config -> 'series') = 'object'
        and best_of_valid(config #> '{series,default}')
        and best_of_valid(config #> '{series,finals}'),
        false)
$$;

alter table tournaments
    add constraint tournaments_starts_at_required
        check (status = 'draft' or starts_at is not null),
    add constraint tournaments_check_in_window
        check (status <> 'check_in' or check_in_minutes is not null),
    add constraint tournaments_champion_rules_shape
        check (champion_rules_valid(champion_rules));

alter table tournament_stages
    add constraint tournament_stages_config_shape check (stage_config_valid(config));

-- Tenant, slug e idioma original do torneio não mudam (o endereço não quebra
-- e as traduções são relativas ao original). Vagas não descem abaixo dos
-- inscritos.
create function tournaments_guard() returns trigger
language plpgsql as $$
begin
    if (new.tenant_id, new.slug, new.locale) is distinct from (old.tenant_id, old.slug, old.locale)
    then
        raise exception 'tenant, slug e idioma original do torneio % não mudam', old.id
            using errcode = 'check_violation', constraint = 'tournaments_identity_immutable';
    end if;
    if new.max_participants < old.max_participants
       and new.max_participants < (select count(*) from participants where tournament_id = new.id)
    then
        raise exception 'torneio % com menos vagas que inscritos', old.id
            using errcode = 'check_violation', constraint = 'tournaments_max_participants_below_registered';
    end if;
    return new;
end $$;

create trigger tournaments_guard
    before update on tournaments
    for each row execute function tournaments_guard();

-- Todo torneio tem ao menos uma fase, conferido no commit: criar insere o
-- torneio e depois as fases, e mudar o formato apaga e insere de novo.
create function check_tournament_stages(tournament bigint) returns void
language plpgsql as $$
begin
    if exists (select 1 from tournaments where id = tournament)
       and not exists (select 1 from tournament_stages where tournament_id = tournament)
    then
        raise exception 'torneio % sem fase', tournament
            using errcode = 'check_violation', constraint = 'tournament_stages_required';
    end if;
end $$;

create function tournaments_stage_required() returns trigger
language plpgsql as $$
begin
    perform check_tournament_stages(new.id);
    return null;
end $$;

create function tournament_stages_required() returns trigger
language plpgsql as $$
begin
    perform check_tournament_stages(old.tournament_id);
    return null;
end $$;

create constraint trigger tournaments_stage_required
    after insert on tournaments
    deferrable initially deferred
    for each row execute function tournaments_stage_required();

create constraint trigger tournament_stages_required
    after delete or update of tournament_id on tournament_stages
    deferrable initially deferred
    for each row execute function tournament_stages_required();

-- 2. Tradução: nunca no idioma original do torneio.
delete from tournament_translations tr
using tournaments t
where t.id = tr.tournament_id and t.locale = tr.locale;

create function tournament_translations_not_original() returns trigger
language plpgsql as $$
begin
    if exists (select 1 from tournaments where id = new.tournament_id and locale = new.locale) then
        raise exception 'tradução no idioma original do torneio %', new.tournament_id
            using errcode = 'check_violation', constraint = 'tournament_translations_not_original';
    end if;
    return new;
end $$;

create trigger tournament_translations_not_original
    before insert or update on tournament_translations
    for each row execute function tournament_translations_not_original();

-- 3. O slug do tenant não muda (links de convite e de torneio).
create function tenants_slug_immutable() returns trigger
language plpgsql as $$
begin
    if new.slug <> old.slug then
        raise exception 'o slug do tenant % não muda', old.id
            using errcode = 'check_violation', constraint = 'tenants_slug_immutable';
    end if;
    return new;
end $$;

create trigger tenants_slug_immutable
    before update of slug on tenants
    for each row execute function tenants_slug_immutable();

-- 4. Inscritos.

-- Vagas: ninguém entra com o torneio lotado (a API trava o torneio antes de
-- contar).
create function participants_capacity() returns trigger
language plpgsql as $$
begin
    if (select count(*) from participants where tournament_id = new.tournament_id)
       >= (select max_participants from tournaments where id = new.tournament_id)
    then
        raise exception 'torneio % lotado', new.tournament_id
            using errcode = 'check_violation', constraint = 'participants_capacity';
    end if;
    return new;
end $$;

create trigger participants_capacity
    before insert on participants
    for each row execute function participants_capacity();

-- O convite usado na inscrição é um link de inscrição deste torneio (o do
-- tenant só faz entrar no tenant). Dado antigo fora disso fica sem convite.
update participants p set invite_id = null
where invite_id is not null
  and not exists (select 1 from tenant_invites i
                  where i.tenant_id = p.tenant_id and i.tournament_id = p.tournament_id
                    and i.id = p.invite_id);

alter table tenant_invites
    add constraint tenant_invites_tenant_id_tournament_id_id_key
        unique (tenant_id, tournament_id, id);

alter table participants
    drop constraint participants_tenant_id_invite_id_fkey,
    add constraint participants_invite_id_fkey
        foreign key (tenant_id, tournament_id, invite_id)
        references tenant_invites (tenant_id, tournament_id, id) on delete set null (invite_id);

-- 5. Usuários e logins.

-- Nome exibido nunca vazio; avatar só por https (é a URL do CDN do Discord).
update users set display_name = 'user-' || id where display_name = '';
update users set avatar_url = null where avatar_url not like 'https://%';
alter table users
    add constraint users_display_name_check check (display_name <> ''),
    add constraint users_avatar_url_https check (avatar_url like 'https://%');

-- Username no formato do cadastro (3 a 20 letras ou números ASCII, _ . -) e
-- senha sempre como hash argon2id.
alter table user_identities
    add constraint user_identities_provider_user_id_check check (provider_user_id <> ''),
    add constraint user_identities_username_format
        check (provider <> 'password' or provider_user_id ~ '^[A-Za-z0-9_.-]{3,20}$'),
    add constraint user_identities_password_hash_format
        check (password_hash is null or password_hash like '$argon2id$%');

-- A chave do limite de tentativas é o username em minúsculas (o login só
-- conta erro de username bem formado). Fora disso não serve para nada: sai.
delete from password_failures where username !~ '^[a-z0-9_.-]{3,20}$';
alter table password_failures
    add constraint password_failures_username_format check (username ~ '^[a-z0-9_.-]{3,20}$');

-- Segredos guardados como SHA-256 (32 bytes). Outro tamanho nunca confere
-- com um token: a linha é inútil e sai.
delete from sessions where length(id) <> 32;
alter table sessions add constraint sessions_id_sha256 check (length(id) = 32);
delete from tenant_invites where length(token_hash) <> 32;
alter table tenant_invites
    add constraint tenant_invites_token_hash_sha256 check (length(token_hash) = 32);

-- 6. Campeões: nome único e não vazio, como os mapas.
alter table champions
    add constraint champions_name_check check (length(name) between 1 and 64),
    add constraint champions_name_key unique (name);
