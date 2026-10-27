-- Campeões do jogo, globais. Campeão novo no jogo = linha nova; o que sai do
-- jogo fica com active = false. O pool de um torneio é o conjunto ativo.
create table champions (
    id     bigint generated always as identity primary key,
    slug   text not null unique check (slug ~ '^[a-z0-9]+(-[a-z0-9]+)*$'),
    name   text not null,
    active boolean not null default true
);

insert into champions (slug, name) values
    ('anarki', 'Anarki'),
    ('athena', 'Athena'),
    ('bj-blazkowicz', 'B.J. Blazkowicz'),
    ('clutch', 'Clutch'),
    ('death-knight', 'Death Knight'),
    ('doom-slayer', 'Doom Slayer'),
    ('eisen', 'Eisen'),
    ('galena', 'Galena'),
    ('keel', 'Keel'),
    ('nyx', 'Nyx'),
    ('ranger', 'Ranger'),
    ('scalebearer', 'Scalebearer'),
    ('slash', 'Slash'),
    ('sorlag', 'Sorlag'),
    ('strogg-peeker', 'Strogg & Peeker'),
    ('visor', 'Visor'),
    ('wrath', 'Wrath');

-- Torneio de um tenant. O slug é o identificador público
-- (/{locale}/{tenant}/{slug}), único no tenant e fixo depois de criado.
-- Textos no idioma original (locale); traduções em tournament_translations.
create table tournaments (
    id               bigint generated always as identity primary key,
    tenant_id        bigint not null references tenants (id) on delete cascade,
    slug             text not null
                     check (slug ~ '^[a-z0-9]+(-[a-z0-9]+)*$' and length(slug) between 3 and 32),
    locale           text not null check (locale in ('pt', 'es')),
    name             text not null check (length(name) between 1 and 80),
    -- Markdown.
    description      text not null default '' check (length(description) <= 2000),
    rules            text not null default '' check (length(rules) <= 20000),
    mode             text not null check (mode in ('duel')),
    status           text not null default 'draft'
                     check (status in ('draft', 'registration', 'check_in', 'in_progress', 'finished')),
    max_participants integer not null check (max_participants between 2 and 1024),
    -- engine::champions::ChampionRules, validado pelo engine antes de gravar.
    champion_rules   jsonb not null,
    created_at       timestamptz not null default now(),
    updated_at       timestamptz not null default now(),
    unique (tenant_id, slug)
);

-- Tradução opcional por idioma (nunca o original, conferido na API). Descrição
-- ou regras vazias mostram as do original.
create table tournament_translations (
    tournament_id bigint not null references tournaments (id) on delete cascade,
    locale        text not null check (locale in ('pt', 'es')),
    name          text not null check (length(name) between 1 and 80),
    description   text not null default '' check (length(description) <= 2000),
    rules         text not null default '' check (length(rules) <= 20000),
    primary key (tournament_id, locale)
);

-- Fases do torneio, na ordem. config é um engine::formats::StageFormat,
-- validado pelo engine antes de gravar; o estado da fase nunca é guardado.
create table tournament_stages (
    id            bigint generated always as identity primary key,
    tournament_id bigint not null references tournaments (id) on delete cascade,
    position      smallint not null check (position >= 1),
    config        jsonb not null,
    unique (tournament_id, position)
);
