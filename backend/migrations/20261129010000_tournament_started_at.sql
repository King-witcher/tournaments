-- Quando o torneio começou (decisões em "Resultados" no docs/decisions.md):
-- o prazo de confirmação de uma partida enviada conta a duração prevista
-- desde que ela foi liberada, e a partida da primeira rodada, sem veto, é
-- liberada no início do torneio.

-- Nulo antes de começar; mantido pelo trigger tournaments_started_at.
alter table tournaments add column started_at timestamptz;

-- Dados existentes: o torneio iniciado começou no primeiro registro da chave
-- (partida enviada ou lançada, evento de veto, série decidida, pareamento do
-- suíço); sem nenhum, quando foi alterado pela última vez. É o mais cedo que
-- se sabe, então o prazo das partidas já enviadas não muda (fica gravado em
-- cada uma) e o das próximas só fica mais curto, nunca mais longo.
update tournaments t
set started_at = least(
    t.updated_at,
    (select min(g.reported_at) from match_games g where g.tournament_id = t.id),
    (select min(e.created_at) from match_veto_events e where e.tournament_id = t.id),
    (select min(r.confirmed_at) from match_results r where r.tournament_id = t.id),
    (select min(p.created_at) from stage_pairings p where p.tournament_id = t.id))
where status in ('in_progress', 'finished');

alter table tournaments
    add constraint tournaments_started_at
        check ((status in ('in_progress', 'finished')) = (started_at is not null));

-- Iniciar marca a hora; encerrar e reabrir (uma correção que volta a deixar
-- partida por jogar) a mantêm; desfazer o início tira a marca, e o próximo
-- início marca de novo.
create function tournaments_started_at() returns trigger
language plpgsql as $$
begin
    if new.status in ('in_progress', 'finished') then
        if tg_op = 'INSERT' or old.status not in ('in_progress', 'finished') then
            new.started_at := coalesce(new.started_at, now());
        end if;
    else
        new.started_at := null;
    end if;
    return new;
end $$;

create trigger tournaments_started_at
    before insert or update of status on tournaments
    for each row execute function tournaments_started_at();
