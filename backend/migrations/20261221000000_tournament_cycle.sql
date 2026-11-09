-- O ciclo do torneio (decisões do usuário em 09/11, docs/decisions/tournaments.md,
-- "Ciclo"): a data de início passa a ser só informação e nada acontece por ela;
-- o check-in é ligado ou desligado e abre só pelo botão da organização, sem
-- janela em minutos; o rascunho é só configuração, sem inscritos; e o aviso
-- "hora de iniciar" (`tournament.start_due`) sai, ficando só o tile na home da
-- organização.
--
-- Dados existentes:
-- - `check_in` = havia janela (`check_in_minutes` não nulo). Torneio em
--   status `check_in` sempre tinha janela (check tournaments_check_in_window).
-- - Rascunho com inscritos (o da organização, que o novo modelo não admite):
--   os inscritos saem, e com eles os times, porque quem foi inscrito num
--   rascunho nem enxerga o torneio. Sem aviso. Nunca começou: nada de
--   resultado, seed ou rating depende deles.
-- - O código `tournament.start_due` some de todo lugar em que está guardado: a
--   notificação cujo código a API não conhece não aparece e continua na
--   contagem das não lidas (`notifications::page`).

-- 1. Os rascunhos ficam sem ninguém. Os times antes dos inscritos: sem o
-- inscrito, o time do duelo fica sem jogador, e o time apagado deixa o
-- inscrito sem time até o `delete` seguinte, dentro da mesma transação.
delete from teams
where tournament_id in (select id from tournaments where status = 'draft');
delete from participants
where tournament_id in (select id from tournaments where status = 'draft');

-- 2. O check-in deixa de ser uma janela.
alter table tournaments add column check_in boolean not null default true;
comment on column tournaments.check_in is
    'Se o torneio tem check-in, que a organização abre pelo botão (status check_in); falso: a inscrição vai direto para em andamento.';
update tournaments set check_in = false where check_in_minutes is null;

-- 3. O aviso de hora de iniciar não sai mais (nem o que já estava para sair).
delete from notifications where notice ->> 'code' = 'tournament.start_due';
delete from outbox where event -> 'notice' ->> 'code' = 'tournament.start_due';
delete from discord_messages where code = 'tournament.start_due';
update discord_webhooks
set events = array_remove(events, 'tournament.start_due')
where 'tournament.start_due' = any(events);

-- As conferências do commit que os `delete` deixaram pendentes rodam já: o
-- Postgres não altera tabela com evento de trigger pendente.
set constraints all immediate;

-- 4. Saem a janela (e, com ela, o check de faixa e tournaments_check_in_window)
-- e o marco do aviso (e tournaments_start_reminded_published).
alter table tournaments
    drop column check_in_minutes,
    drop column start_reminded_at,
    add constraint tournaments_check_in_enabled check (status <> 'check_in' or check_in);

-- 5. Rascunho sem inscritos, nos dois sentidos. O `for share` e o lock da
-- atualização do status põem em fila uma inscrição e uma volta ao rascunho ao
-- mesmo tempo, e a segunda enxerga o que a primeira gravou (a API já trava o
-- torneio antes; isto vale mesmo que ela erre).
create function participants_not_in_draft() returns trigger
language plpgsql as $$
begin
    if (select status from tournaments where id = new.tournament_id for share) = 'draft' then
        raise exception 'torneio % é rascunho e não tem inscritos', new.tournament_id
            using errcode = 'check_violation', constraint = 'participants_not_in_draft';
    end if;
    return new;
end $$;

create trigger participants_not_in_draft
    before insert on participants
    for each row execute function participants_not_in_draft();

create function tournaments_draft_without_participants() returns trigger
language plpgsql as $$
begin
    if exists (select 1 from participants where tournament_id = new.id) then
        raise exception 'torneio % tem inscritos e não volta a rascunho', new.id
            using errcode = 'check_violation', constraint = 'tournaments_draft_without_participants';
    end if;
    return new;
end $$;

create trigger tournaments_draft_without_participants
    before update of status on tournaments
    for each row when (new.status = 'draft' and old.status <> 'draft')
    execute function tournaments_draft_without_participants();
