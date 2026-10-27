-- Visibilidade: público aparece para todos os membros do tenant; privado só
-- para inscritos, organizers e owner.
alter table tournaments
    add column visibility text not null default 'public'
        check (visibility in ('public', 'private')),
    -- Alvo das FKs compostas de inscritos e convites: sempre do mesmo tenant.
    add unique (tenant_id, id);

-- Convite de torneio: além de entrar no tenant, inscreve no torneio. Vale
-- enquanto a inscrição estiver aberta, sem data fixa; o de tenant continua
-- com validade.
alter table tenant_invites
    add column tournament_id bigint,
    alter column expires_at drop not null,
    add foreign key (tenant_id, tournament_id) references tournaments (tenant_id, id)
        on delete cascade,
    add check ((tournament_id is null) = (expires_at is not null));

create index tenant_invites_tournament_id_idx on tenant_invites (tournament_id)
    where tournament_id is not null;

-- Inscritos de um torneio. Só membro do tenant se inscreve; sair do tenant
-- tira das inscrições. O id é o ParticipantId da chave.
create table participants (
    id            bigint generated always as identity primary key,
    tenant_id     bigint not null,
    tournament_id bigint not null,
    user_id       bigint not null,
    -- Convite usado (nulo: autoinscrição ou inserção pelo organizador).
    invite_id     bigint,
    -- Organizador que inscreveu (nulo: a própria pessoa, sozinha ou por convite).
    added_by      bigint references users (id) on delete set null,
    registered_at timestamptz not null default now(),
    unique (tournament_id, user_id),
    foreign key (tenant_id, tournament_id) references tournaments (tenant_id, id)
        on delete cascade,
    foreign key (tenant_id, user_id) references tenant_members (tenant_id, user_id)
        on delete cascade,
    foreign key (tenant_id, invite_id) references tenant_invites (tenant_id, id)
        on delete set null (invite_id)
);

-- Torneios em que um membro está inscrito (visibilidade dos privados).
create index participants_member_idx on participants (tenant_id, user_id);
