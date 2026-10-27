-- Tenant ("comunidade" na interface). O slug é o identificador público
-- (/{locale}/{slug}) e não muda depois de criado; o id só serve de chave.
create table tenants (
    id         bigint generated always as identity primary key,
    slug       text not null unique
               check (slug ~ '^[a-z0-9]+(-[a-z0-9]+)*$' and length(slug) between 3 and 32),
    name       text not null check (length(name) between 1 and 64),
    created_at timestamptz not null default now()
);

-- Link de convite multiuso. Só o SHA-256 do token fica guardado; quem entra
-- por ele vira player. Revogar marca revoked_at (a linha fica para o histórico).
create table tenant_invites (
    id         bigint generated always as identity primary key,
    tenant_id  bigint not null references tenants (id) on delete cascade,
    token_hash bytea not null unique,
    created_by bigint not null references users (id) on delete cascade,
    created_at timestamptz not null default now(),
    expires_at timestamptz not null,
    revoked_at timestamptz,
    -- Alvo da FK composta de tenant_members: convite e membro sempre do mesmo tenant.
    unique (tenant_id, id)
);

create table tenant_members (
    tenant_id bigint not null references tenants (id) on delete cascade,
    user_id   bigint not null references users (id) on delete cascade,
    role      text not null check (role in ('owner', 'organizer', 'player')),
    nickname  text not null check (length(nickname) between 1 and 32),
    -- Convite usado para entrar (nulo para o owner).
    invite_id bigint,
    joined_at timestamptz not null default now(),
    primary key (tenant_id, user_id),
    foreign key (tenant_id, invite_id) references tenant_invites (tenant_id, id)
        on delete set null (invite_id)
);

-- "Meus tenants" e a contagem do limite de tenants por owner.
create index tenant_members_user_id_idx on tenant_members (user_id, role);
-- Um owner por tenant.
create unique index tenant_members_one_owner_idx on tenant_members (tenant_id) where role = 'owner';
