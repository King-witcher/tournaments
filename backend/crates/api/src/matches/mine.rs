//! Onde quem vê está na chave de um torneio e qual é a próxima partida dele,
//! para a lista dos torneios dele em todas as comunidades (`/me/tournaments`),
//! e onde outro inscrito está, sem a próxima partida, para o perfil do membro
//! na comunidade (`place_of`).

use engine::{
    core::{self as ec, Occupant, ParticipantId, Slot},
    formats::{self as ef, elimination::EliminationKind},
};
use sqlx::PgConnection;

use super::{
    due, key,
    result::Side,
    stage::{self, Stage},
    tasks,
    types::{MatchState, NextMatch, SideSource, SourceOutcome},
    view::{self, Viewer},
};
use crate::{
    shared::ApiError,
    tenants::TenantMember,
    tournaments::{StoredTournament, TournamentStatus},
};

/// Onde o inscrito está na chave. A colocação é a faixa (`rank` 5 com
/// `tied` 4 é 5º–8º).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    /// Inscrito sem seed: o torneio começou sem ele (não fez check-in).
    Missed,
    /// Na chave e ainda vivo.
    Playing,
    /// Desistiu; com a colocação quando ela já se definiu.
    Withdrawn(Option<(u16, u16)>),
    /// Eliminado, com a colocação.
    Eliminated(u16, u16),
    Champion,
}

pub struct BracketPlace {
    /// O torneio depois das confirmações por tempo (pode ter encerrado).
    pub tournament: StoredTournament,
    pub place: Place,
    /// Só de quem ainda joga, e só em `place` (quem vê).
    pub next: Option<NextMatch>,
}

/// O lugar de `member` (inscrito) na chave do torneio `slug`, em andamento ou
/// encerrado, depois das confirmações por tempo vencidas.
pub async fn place<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    slug: &str,
) -> Result<BracketPlace, ApiError> {
    let tournament = due::find_current(conn, member, slug).await?;
    let stage = stage::load(conn, &tournament).await?;
    let viewer = Viewer::new(member, &stage);
    let place = locate(&stage, viewer.participant);
    let next = match place {
        Place::Playing => next_match(tournament.status, &stage, &viewer),
        _ => None,
    };
    Ok(BracketPlace {
        tournament,
        place,
        next,
    })
}

/// Como `place`, para outro inscrito, `user_id`, no torneio `slug` como
/// `member` o vê: sem a próxima partida, que é só de quem joga.
pub async fn place_of<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    slug: &str,
    user_id: i64,
) -> Result<BracketPlace, ApiError> {
    let tournament = due::find_current(conn, member, slug).await?;
    let stage = stage::load(conn, &tournament).await?;
    let place = locate(&stage, stage.participant_of(user_id));
    Ok(BracketPlace {
        tournament,
        place,
        next: None,
    })
}

/// O lugar de `participant` na chave; `None` é o inscrito sem seed.
fn locate(stage: &Stage, participant: Option<ParticipantId>) -> Place {
    let Some(me) = participant else {
        return Place::Missed;
    };
    let standing = stage
        .bracket
        .standings()
        .into_iter()
        .find(|p| p.participant == me)
        .map(|p| (p.rank, p.tied));
    let withdrawn = stage.entrant(me).is_some_and(|e| e.withdrawn);
    match standing {
        Some((1, _)) => Place::Champion,
        _ if withdrawn => Place::Withdrawn(standing),
        Some((rank, tied)) => Place::Eliminated(rank, tied),
        None => Place::Playing,
    }
}

/// A partida pronta de quem vê ou, sem nenhuma, a primeira em que ele já
/// está e que espera o adversário sair de outra.
fn next_match(status: TournamentStatus, stage: &Stage, viewer: &Viewer) -> Option<NextMatch> {
    let mine = |ready: bool| {
        stage.bracket.matches().iter().find(|m| {
            let wanted = match m.state {
                ec::MatchState::Ready { .. } => ready,
                ec::MatchState::Pending => !ready,
                _ => false,
            };
            wanted && view::viewer_side(stage, m, viewer).is_some()
        })
    };
    let m = mine(true).or_else(|| mine(false))?;
    let side = view::viewer_side(stage, m, viewer)?;
    let (oa, ob) = view::occupants(stage, m);
    let (opponent, slot) = match side {
        Side::A => (ob, m.b),
        Side::B => (oa, m.a),
    };
    let ef::StageFormat::Elimination(config) = stage.format;
    Some(NextMatch {
        key: key::format(m.id),
        state: match m.state {
            ec::MatchState::Ready { .. } => MatchState::Ready,
            _ => MatchState::Pending,
        },
        single: config.kind == EliminationKind::Single,
        opponent: match opponent {
            Occupant::Participant(p) => stage.entrant(p).map(|e| e.nickname.clone()),
            _ => None,
        },
        opponent_from: match (opponent, slot) {
            (Occupant::Unknown, Slot::WinnerOf(id)) => Some(SideSource {
                outcome: SourceOutcome::Winner,
                match_key: key::format(id),
            }),
            (Occupant::Unknown, Slot::LoserOf(id)) => Some(SideSource {
                outcome: SourceOutcome::Loser,
                match_key: key::format(id),
            }),
            _ => None,
        },
        task: tasks::match_task(status, m, &stage.games_of(m.id), Some(side), viewer),
    })
}
