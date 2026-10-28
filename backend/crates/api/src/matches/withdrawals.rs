//! W.O. automático: toda partida pronta com um lado fora do torneio (desistiu
//! ou saiu da comunidade) vira W.O. para o adversário, confirmada e sem
//! autor, com o evento `auto_walkover` no histórico. Aplicado depois de toda
//! mudança que pode deixar partida pronta (confirmação, inclusive por tempo,
//! resolução, lançamento, correção, W.O. e a própria desistência), então
//! nenhuma partida pronta fica parada esperando quem saiu. Um envio pendente
//! ou contestado da partida é substituído pelo W.O.

use engine::core::{MatchId, MatchState, Outcome, ParticipantId};
use sqlx::PgConnection;

use super::{
    db::{self, MatchRef},
    history::NewEvent,
    key,
    stage::{self, Stage},
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

/// Dá os W.O. automáticos até não sobrar partida pronta com quem saiu: um
/// W.O. pode deixar pronta outra partida dele (na dupla, quem perde na chave
/// de cima cai na de baixo). Quem chama trava o torneio. Devolve a fase como
/// ficou.
pub async fn apply(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
) -> Result<Stage, ApiError> {
    let mut stage = stage::load(conn, tournament).await?;
    loop {
        let due = due(&stage);
        if due.is_empty() {
            return Ok(stage);
        }
        for d in due {
            let key = key::format(d.id);
            let at = MatchRef {
                tenant: tournament.tenant,
                tournament: tournament.id,
                stage: stage.number,
                key: &key,
            };
            let id = db::put_walkover(conn, &at, d.best_of, d.sides, d.winner).await?;
            db::replace_games(conn, id, &[]).await?;
            let event = NewEvent {
                kind: EventKind::AutoWalkover,
                actor: None,
                outcome: Some(Outcome::Walkover { winner: d.winner }),
                comment: None,
            };
            db::insert_event(conn, &at, &event).await?;
        }
        stage = stage::load(conn, tournament).await?;
    }
}

/// Para quem muda o torneio fora de `matches` (a desistência): aplica os W.O.
/// automáticos e encerra o torneio se a chave terminou. Quem chama trava o
/// torneio, que está em andamento.
pub async fn settle(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
) -> Result<TournamentStatus, ApiError> {
    let stage = apply(conn, tournament).await?;
    stage::sync_status(conn, tournament, &stage).await
}
