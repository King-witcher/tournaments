//! O que a chave anuncia só ao canal do Discord da comunidade
//! (`Event::Announce`): o torneio ou uma fase dele começou, o torneio
//! terminou (com o pódio da classificação final) ou uma série foi decidida
//! ou empatou. Quem recebe e em que texto é do domínio `discord`.

use engine::core::{Match, MatchState};
use sqlx::PgConnection;

use super::{key, stage::Stage};
use crate::{
    outbox::{self, Announce, AnnounceKind, Event, Placement},
    tournaments::Address,
};

/// O pódio de quem fica até o 3º lugar.
const PODIUM: u16 = 3;

/// A chave da fase `stage` foi gerada: a 1 é o torneio que começou.
pub async fn started(
    conn: &mut PgConnection,
    audience: &Address,
    stage: i16,
) -> Result<(), sqlx::Error> {
    let tournament = audience.name.clone();
    let kind = if stage > 1 {
        AnnounceKind::StageStarted { tournament, stage }
    } else {
        AnnounceKind::TournamentStarted { tournament }
    };
    emit(conn, kind, audience.path()).await
}

/// A última fase terminou: o pódio na ordem da classificação final do
/// torneio (`standings`, com os das fases anteriores).
pub async fn finished(
    conn: &mut PgConnection,
    audience: &Address,
    stage: &Stage,
    standings: &[engine::core::Placement],
) -> Result<(), sqlx::Error> {
    let team = |p: engine::core::ParticipantId| stage.teams.iter().find(|t| t.id == p.0);
    let podium = standings
        .iter()
        .filter(|p| p.rank <= PODIUM)
        .filter_map(|p| {
            Some(Placement {
                rank: u32::from(p.rank),
                player: team(p.participant)?.display_name(),
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
    let key = key::public(stage.number, m.id);
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
