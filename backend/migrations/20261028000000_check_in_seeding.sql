-- Data de início, check-in e seeding (decisões em "Torneios" no
-- docs/decisions.md). Ciclo depois de publicar: inscrição → check-in (abre
-- sozinho starts_at - check_in_minutes, conferido pela API ao ler) → em
-- andamento, que o organizador inicia à mão.
alter table tournaments
    -- Início previsto. Rascunho pode ficar sem; publicar exige (conferido na
    -- API, não aqui: torneios publicados antes desta migration não têm).
    add column starts_at timestamptz,
    -- Minutos antes do início em que o check-in abre sozinho. Nulo: sem
    -- check-in (a inscrição vai direto para em andamento).
    add column check_in_minutes integer default 30
        check (check_in_minutes between 5 and 1440),
    -- Quando o organizador reabriu a inscrição durante o check-in. A abertura
    -- automática só volta a valer para uma abertura (starts_at - janela)
    -- depois deste momento: mudar a data ou a janela reativa.
    add column registration_reopened_at timestamptz,
    -- Seed do último sorteio de seeding (o u64 do engine gravado bit a bit
    -- num bigint): reproduz a ordem sorteada.
    add column seeding_seed bigint;

alter table participants
    -- Check-in feito (nulo: não fez). Quem não fez continua inscrito, mas fica
    -- fora do seeding e da chave.
    add column checked_in_at timestamptz,
    -- Organizador que marcou o check-in (nulo: o próprio inscrito).
    add column checked_in_by bigint references users (id) on delete set null,
    -- Posição no seeding (1 = melhor). Ao iniciar, vira 1..N entre os
    -- elegíveis e os outros ficam sem.
    add column seed integer check (seed >= 1),
    add unique (tournament_id, seed),
    add check (checked_in_by is null or checked_in_at is not null);
