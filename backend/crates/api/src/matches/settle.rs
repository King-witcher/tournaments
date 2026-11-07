//! Depois de toda mudança na chave: anula os mapas que ficaram depois da
//! decisão da série (ou de outros lados), fecha as séries que os mapas
//! confirmados decidiram e dá os W.O. automáticos, repetindo até não sobrar
//! nada (fechar uma série pode deixar pronta uma partida de quem desistiu),
//! grava o pareamento da rodada seguinte do suíço quando a anterior termina
//! (`rounds`) e começa o jogo seguinte do veto de cada partida
//! (`veto::advance`); depois, se a fase seguinte já começou, confere que
//! quem entrou nela e a ordem continuam os mesmos (`chain::check_next`; senão,
//! 409 e nada fica gravado), marca o fim da fase e encerra ou reabre o
//! torneio conforme a chave da última (que espera a organização decidir os
//! empates do fim, `ties`), acerta a marca dessa espera e emite o que mudou
//! (`changes`). Quem chama trava o torneio. Tudo na fase da mudança.

use chrono::Utc;
use engine::{core::MatchState, series};
use sqlx::PgConnection;

use super::{
    chain,
    changes::{self, Snapshot},
    db::{self, MatchRef},
    games::{GameStatus, StoredGame, series_games},
    history::NewEvent,
    key,
    result::{Side, series_outcome},
    rounds,
    stage::{self, Stage},
    ties,
    types::EventKind,
    veto, withdrawals,
};
use crate::{
    shared::ApiError,
    tournaments::{StoredTournament, TournamentStatus},
};

/// Aplica tudo e devolve a fase como ficou. `actor`: quem fez a mudança (fica
/// como autor dos mapas anulados). Começa por `first`, a fase já lida depois
/// da mudança (sem ela, lê); a cada passo que grava, lê a fase inteira de
/// novo.
async fn apply_from(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    number: i16,
    actor: Option<i64>,
    mut first: Option<Stage>,
) -> Result<Stage, ApiError> {
    loop {
        let stage = match first.take() {
            Some(stage) => stage,
            None => stage::load(conn, tournament, number).await?,
        };
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

/// `apply_from`, o status do torneio conforme a chave (encerra quando ela termina,
/// reabre quando uma correção deixa partida por jogar) e o que mudou desde
/// `before` no outbox (`changes`). Devolve a fase e o status como ficaram.
pub async fn settle(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    actor: Option<i64>,
    before: &Snapshot,
) -> Result<(Stage, TournamentStatus), ApiError> {
    settle_from(conn, tournament, actor, before, None).await
}

/// `settle` a partir de `first`, a fase já lida depois da mudança
/// (`Stage::refresh`); sem ela, lê.
pub async fn settle_from(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    actor: Option<i64>,
    before: &Snapshot,
    first: Option<Stage>,
) -> Result<(Stage, TournamentStatus), ApiError> {
    let mut stage = apply_from(conn, tournament, before.stage(), actor, first).await?;
    chain::check_next(tournament, &stage)?;
    let status = stage::sync_status(conn, tournament, &mut stage).await?;
    ties::sync(conn, tournament, &stage).await?;
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

/// Os mapas que um W.O. anula: do primeiro sem confirmar em diante, também os
/// confirmados depois dele. A contestação não trava o envio do mapa seguinte,
/// então um confirmado pode vir depois de um sem confirmar, e os números vão de
/// 1 a n sem buraco (`match_games_sequence`).
pub fn unsettled<'a>(games: &[&'a StoredGame]) -> Vec<&'a StoredGame> {
    games
        .iter()
        .copied()
        .skip_while(|g| g.status == GameStatus::Confirmed)
        .collect()
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
        let winner = d.winner.map(|w| Side::from(w).of(*sides));
        let score = (d.a_wins, d.b_wins);
        db::put_decided(conn, &at, *best_of, *sides, winner, score, *at_time).await?;
        let outcome = series_outcome(winner, Some(score));
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
