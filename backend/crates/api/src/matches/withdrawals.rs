//! W.O. automático: toda partida pronta com um lado fora do torneio (desistiu
//! ou saiu da comunidade) vira W.O. para o adversário, decidida e sem autor,
//! com o evento `auto_walkover` no histórico. Aplicado depois de toda
//! mudança que pode deixar partida pronta (`settle`), então nenhuma partida
//! pronta fica parada esperando quem saiu. Os mapas confirmados da série
//! ficam (foram jogados); os pendentes e contestados são anulados.

use engine::core::{MatchId, MatchState, Outcome, ParticipantId};
use sqlx::PgConnection;

use super::{
    db::{self, MatchRef},
    games::{GameStatus, StoredGame},
    history::NewEvent,
    key, settle,
    stage::Stage,
    types::EventKind,
};
use crate::{
    shared::ApiError,
    tournaments::{StoredTournament, TournamentStatus},
};

/// Uma partida que vira W.O. automático.
struct Due {
    id: MatchId,
    best_of: u8,
    sides: (ParticipantId, ParticipantId),
    winner: ParticipantId,
}

/// As partidas prontas com um lado fora. O vencedor é o outro lado; com os
/// dois fora, o de seed melhor (que, na dupla, ainda pode cair na chave de
/// baixo e levar outro W.O.).
fn due(stage: &Stage) -> Vec<Due> {
    let out = |p: ParticipantId| stage.entrant(p).map(|e| (e.out(), e.seed));
    stage
        .bracket
        .matches()
        .iter()
        .filter_map(|m| {
            let MatchState::Ready { a, b } = m.state else {
                return None;
            };
            let ((a_out, a_seed), (b_out, b_seed)) = (out(a)?, out(b)?);
            let winner = match (a_out, b_out) {
                (false, false) => return None,
                (true, false) => b,
                (false, true) => a,
                (true, true) if a_seed <= b_seed => a,
                (true, true) => b,
            };
            Some(Due {
                id: m.id,
                best_of: m.best_of,
                sides: (a, b),
                winner,
            })
        })
        .collect()
}

/// Dá os W.O. automáticos das partidas prontas de `stage` com quem saiu.
/// Devolve se deu algum (um W.O. pode deixar pronta outra partida dele: quem
/// chama refaz a fase e chama de novo).
pub async fn give(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    stage: &Stage,
) -> Result<bool, ApiError> {
    let due = due(stage);
    for d in &due {
        let key = key::format(d.id);
        let at = MatchRef {
            tenant: tournament.tenant,
            tournament: tournament.id,
            stage: stage.number,
            key: &key,
        };
        let open: Vec<&StoredGame> = stage
            .games_of(d.id)
            .into_iter()
            .filter(|g| g.status != GameStatus::Confirmed)
            .collect();
        settle::void(conn, &at, &open, None).await?;
        db::put_walkover(conn, &at, d.best_of, d.sides, d.winner).await?;
        let outcome = Outcome::Walkover { winner: d.winner };
        let event = NewEvent::series(EventKind::AutoWalkover, None, Some(outcome));
        db::insert_event(conn, &at, &event).await?;
    }
    Ok(!due.is_empty())
}

/// Para quem muda o torneio fora de `matches` (a desistência): aplica os W.O.
/// automáticos (e o resto de `settle`) e encerra o torneio se a chave
/// terminou. Quem chama trava o torneio, que está em andamento.
pub async fn settle(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
) -> Result<TournamentStatus, ApiError> {
    settle::settle(conn, tournament, None).await
}
