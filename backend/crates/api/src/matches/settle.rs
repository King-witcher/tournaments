//! Depois de toda mudança na chave: anula os mapas que ficaram depois da
//! decisão da série (ou de outros lados), fecha as séries que os mapas
//! confirmados decidiram e dá os W.O. automáticos, repetindo até não sobrar
//! nada (fechar uma série pode deixar pronta uma partida de quem desistiu),
//! grava o pareamento da rodada seguinte do suíço quando a anterior termina
//! (`rounds`) e começa o jogo seguinte do veto de cada partida
//! (`veto::advance`); depois encerra ou reabre o torneio conforme a chave e
//! emite o que mudou (`changes`). Quem chama trava o torneio.

use chrono::Utc;
use engine::{core::MatchState, series};
use sqlx::PgConnection;

use super::{
    changes::{self, Snapshot},
    db::{self, MatchRef},
    games::{StoredGame, series_games},
    history::NewEvent,
    key,
    result::played_or_walkover,
    rounds,
    stage::{self, Stage},
    types::EventKind,
    veto, withdrawals,
};
use crate::{
    shared::ApiError,
    tournaments::{StoredTournament, TournamentStatus},
};

/// Aplica tudo e devolve a fase como ficou. `actor`: quem fez a mudança (fica
/// como autor dos mapas anulados).
pub async fn apply(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    actor: Option<i64>,
) -> Result<Stage, ApiError> {
    loop {
        let stage = stage::load(conn, tournament).await?;
        if close_series(conn, tournament, &stage, actor).await? {
            continue;
        }
        if withdrawals::give(conn, tournament, &stage).await? {
            continue;
        }
        if rounds::pair_next(conn, tournament, &stage).await? {
            continue;
        }
        if veto::advance(conn, tournament, &stage).await? {
            continue;
        }
        return Ok(stage);
    }
}

/// `apply`, o status do torneio conforme a chave (encerra quando ela termina,
/// reabre quando uma correção deixa partida por jogar) e o que mudou desde
/// `before` no outbox (`changes`). Devolve a fase e o status como ficaram.
pub async fn settle(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    actor: Option<i64>,
    before: &Snapshot,
) -> Result<(Stage, TournamentStatus), ApiError> {
    let stage = apply(conn, tournament, actor).await?;
    let status = stage::sync_status(conn, tournament, &stage).await?;
    changes::settled(conn, tournament, before, &stage, status).await?;
    Ok((stage, status))
}

/// Anula os mapas `games` da partida `at`, com um evento por mapa.
pub async fn void(
    conn: &mut PgConnection,
    at: &MatchRef<'_>,
    games: &[&StoredGame],
    actor: Option<i64>,
) -> Result<(), ApiError> {
    let ids: Vec<i64> = games.iter().map(|g| g.id).collect();
    db::void_games(conn, &ids).await?;
    for g in games {
        let event = NewEvent::game(EventKind::Voided, actor, g.number, None);
        db::insert_event(conn, at, &event).await?;
    }
    Ok(())
}

/// Nas partidas prontas sem série decidida: anula os mapas que não valem e
/// fecha as séries que os mapas confirmados decidiram, na ordem em que
/// decidiram (é a ordem do replay). Devolve se mudou alguma coisa.
async fn close_series(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    stage: &Stage,
    actor: Option<i64>,
) -> Result<bool, ApiError> {
    let mut voided = false;
    let mut decided = Vec::new();
    for m in stage.bracket.matches() {
        let MatchState::Ready { a, b } = m.state else {
            continue;
        };
        let games = stage.games_of(m.id);
        if games.is_empty() {
            continue;
        }
        let key = key::format(m.id);
        let state = series::evaluate(m.best_of, &series_games(&games));
        let stale: Vec<&StoredGame> = games
            .iter()
            .enumerate()
            .filter(|(i, g)| *i >= state.standing || g.sides() != (a, b))
            .map(|(_, g)| *g)
            .collect();
        if !stale.is_empty() {
            let at = match_ref(tournament, stage, &key);
            void(conn, &at, &stale, actor).await?;
            voided = true;
            continue;
        }
        if let Some(d) = state.decided {
            // A série fecha quando confirma o último mapa que faltava.
            let at_time = games
                .iter()
                .filter_map(|g| g.confirmed_at)
                .max()
                .unwrap_or_else(Utc::now);
            decided.push((at_time, key, m.best_of, (a, b), d));
        }
    }
    if voided {
        return Ok(true);
    }
    decided.sort_by(|x, y| (x.0, &x.1).cmp(&(y.0, &y.1)));
    for (at_time, key, best_of, sides, d) in &decided {
        let at = match_ref(tournament, stage, key);
        let winner = super::result::Side::from(d.winner).of(*sides);
        let score = (d.a_wins, d.b_wins);
        db::put_decided(conn, &at, *best_of, *sides, winner, score, *at_time).await?;
        let outcome = played_or_walkover(winner, Some(score));
        let event = NewEvent {
            at: Some(*at_time),
            ..NewEvent::series(EventKind::Decided, None, Some(outcome))
        };
        db::insert_event(conn, &at, &event).await?;
    }
    Ok(!decided.is_empty())
}

fn match_ref<'a>(tournament: &StoredTournament, stage: &Stage, key: &'a str) -> MatchRef<'a> {
    MatchRef {
        tenant: tournament.tenant,
        tournament: tournament.id,
        stage: stage.number,
        key,
    }
}
