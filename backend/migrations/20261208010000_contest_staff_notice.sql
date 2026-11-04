-- Quando avisar a organização de uma partida contestada. O capitão adversário é
-- avisado na hora e pode aceitar; se a partida continuar contestada nesta hora, o
-- agendador avisa a organização e limpa a coluna. Nulo: nada a avisar (já avisou,
-- ou não está contestada). As contestadas de antes ficam nulas: já foram avisadas.
alter table match_games add column contest_notify_at timestamptz;

alter table match_games
    add constraint match_games_contest_notify
    check (contest_notify_at is null or status = 'contested');

-- O agendador procura a cada 10 s.
create index match_games_contest_notify_at on match_games (contest_notify_at)
    where contest_notify_at is not null;
