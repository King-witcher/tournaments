//! O que a chave anuncia só ao canal do Discord da comunidade
//! (`Event::Announce`): o torneio começou, terminou (com o pódio) ou uma série
//! foi decidida ou empatou. Quem recebe e em que texto é do domínio `discord`.

use engine::core::{Match, MatchState};
use sqlx::PgConnection;

use super::{key, stage::Stage};
use crate::{
    outbox::{self, Announce, AnnounceKind, Event, Placement},
    tournaments::Address,
};

/// O pódio de quem fica até o 3º lugar.
const PODIUM: u16 = 3;

/// A chave foi gerada.
pub async fn started(conn: &mut PgConnection, audience: &Address) -> Result<(), sqlx::Error> {
    let kind = AnnounceKind::TournamentStarted {
        tournament: audience.name.clone(),
    };
    emit(conn, kind, audience.path()).await
}

/// A chave terminou: o pódio na ordem da classificação.
pub async fn finished(
    conn: &mut PgConnection,
    audience: &Address,
    stage: &Stage,
) -> Result<(), sqlx::Error> {
    let podium = stage
        .bracket
        .placements()
        .into_iter()
        .filter(|p| p.rank <= PODIUM)
        .filter_map(|p| {
            Some(Placement {
                rank: u32::from(p.rank),
                player: stage.entrant(p.participant)?.display_name(),
            })
        })
        .collect();
    let kind = AnnounceKind::TournamentFinished {
        tournament: audience.name.clone(),
        podium,
    };
    emit(conn, kind, audience.path()).await
}

/// A série da partida `m` foi decidida (ou corrigida): quem venceu, contra
/// quem e o placar em mapas, com o vencedor primeiro; ou, sem vencedor, os dois
/// lados e o placar.
pub async fn series_decided(
    conn: &mut PgConnection,
    audience: &Address,
    stage: &Stage,
    m: &Match,
) -> Result<(), sqlx::Error> {
    let (Some(result), MatchState::Completed { a, b, .. }) = (stage.result(m.id), m.state) else {
        return Ok(());
    };
    let key = key::format(m.id);
    let path = format!("{}/matches/{key}", audience.path());
    let Some(won) = result.winner else {
        let (Some(a), Some(b)) = (stage.entrant(a), stage.entrant(b)) else {
            return Ok(());
        };
        let kind = AnnounceKind::SeriesDrawn {
            tournament: audience.name.clone(),
            match_key: key,
            side_a: a.display_name(),
            side_b: b.display_name(),
            score: result.score.map(|(x, y)| format!("{x}-{y}")),
        };
        return emit(conn, kind, path).await;
    };
    let loser = if won == a { b } else { a };
    let (Some(winner), Some(loser)) = (stage.entrant(won), stage.entrant(loser)) else {
        return Ok(());
    };
    // O placar é de `side_a` e `side_b`; o vencedor vai primeiro.
    let score = result.score.map(|(x, y)| {
        let (first, second) = if won == result.side_a { (x, y) } else { (y, x) };
        format!("{first}-{second}")
    });
    let kind = AnnounceKind::SeriesDecided {
        tournament: audience.name.clone(),
        match_key: key,
        winner: winner.display_name(),
        loser: loser.display_name(),
        score,
    };
    emit(conn, kind, path).await
}

async fn emit(
    conn: &mut PgConnection,
    kind: AnnounceKind,
    path: String,
) -> Result<(), sqlx::Error> {
    let announce = Announce { kind, path };
    outbox::emit(conn, Event::Announce { announce }).await
}
