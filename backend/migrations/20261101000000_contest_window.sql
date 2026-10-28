-- Janela de contestação padrão de 10 minutos (era 5), por mapa (decisões em
-- "Resultados" no docs/decisions.md).
--
-- Torneios que ainda não começaram e estão com 5 minutos, o padrão antigo,
-- passam para 10: 5 era o que o formulário trazia marcado, então quase sempre
-- é o padrão e não uma escolha. Torneios em andamento ou encerrados ficam
-- como estão (o prazo de cada envio já fica gravado nele). Só muda a coluna
-- contest_minutes, que o guarda de tournaments não confere.
alter table tournaments alter column contest_minutes set default 10;

update tournaments
set contest_minutes = 10
where contest_minutes = 5 and status in ('draft', 'registration', 'check_in');
