//! Onde quem vê está num torneio e qual é a próxima partida dele, para a
//! lista dos torneios dele em todas as comunidades (`/me/tournaments`) e para
//! a entrada de um torneio só; onde ele terminou nos encerrados, de vários de
//! uma vez (`final_places`); e onde outro inscrito está, sem a próxima
//! partida, para o perfil do membro na comunidade (`place_of`).
//!
//! Com fases ligadas: na fase atual, a colocação dela só vale como a do
//! torneio na última fase; entre as fases (a atual terminou, a seguinte não
//! começou), quem jogou a atual espera a seguinte, confirma presença ou é
//! suplente (`chain::projection`); quem ficou numa fase anterior tem a
//! posição da classificação final (`chain::standings`).

use chrono::Utc;
use engine::{
    core::{self as ec, Occupant, ParticipantId, Placement, Slot},
    formats::{self as ef, StageBracket, elimination::EliminationKind},
};
use sqlx::PgConnection;

use super::{
    chain, due, key, progress,
    result::Side,
    stage::{self, Stage},
    tasks,
    types::{MatchState, NextMatch, SideSource, SourceOutcome, ViewerTask},
    veto,
    view::{self, Viewer},
};
use crate::{
    shared::ApiError,
    teams::StoredTeam,
    tenants::TenantMember,
    tournaments::{StoredTournament, TournamentId, TournamentStatus, find_visible},
};

/// Onde o inscrito está na chave. A colocação é a faixa (`rank` 5 com
/// `tied` 4 é 5º–8º).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    /// Inscrito sem seed: o torneio começou sem ele (não fez check-in).
    Missed,
    /// Na chave e ainda vivo (no round robin e no suíço, até a fase terminar:
    /// a colocação sai da tabela final; antes da última fase, até ela
    /// terminar).
    Playing,
    /// Desistiu; com a colocação quando ela já se definiu.
    Withdrawn(Option<(u16, u16)>),
    /// Num empate do fim da fase que espera a organização: a melhor posição
    /// do grupo e quantos estão nele.
    TiePending(u16, u16),
    /// Eliminado, ou fora do 1º lugar na classificação final, com a
    /// colocação.
    Eliminated(u16, u16),
    Champion,
    /// Entre as fases: está entre os que entram na seguinte.
    AwaitingStage,
    /// Entre as fases, com o check-in da seguinte aberto: falta confirmar a
    /// presença (sem ela, fica fora).
    StageCheckIn,
    /// Entre as fases: abaixo do corte, entra se alguém de cima não confirmar
    /// ou desistir.
    Substitute,
}

pub struct BracketPlace {
    /// O torneio depois das confirmações por tempo (pode ter encerrado).
    pub tournament: StoredTournament,
    pub place: Place,
    /// A fase de que o lugar fala: a atual, ou a seguinte para quem espera
    /// por ela (`AwaitingStage`, `StageCheckIn`, `Substitute`).
    pub stage: i16,
    /// Só de quem ainda joga, e só em `place` (quem vê).
    pub next: Option<NextMatch>,
}

/// O lugar de `member` (inscrito) no torneio `slug`, em andamento ou
/// encerrado, depois das confirmações por tempo vencidas.
pub async fn place<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    slug: &str,
) -> Result<BracketPlace, ApiError> {
    let tournament = find_visible(conn, member, slug).await?;
    place_from(conn, member, tournament).await
}

/// Como `place`, com o torneio já lido por `find_visible`.
pub async fn place_from<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    tournament: StoredTournament,
) -> Result<BracketPlace, ApiError> {
    let (tournament, stage) = due::current_from(conn, member, tournament, None).await?;
    let viewer = Viewer::new(member, &stage);
    let me = stage::participant_of(&stage.teams, member.user.id);
    let (place, number) = locate(conn, &tournament, &stage, me).await?;
    let next = match place {
        Place::Playing => next_match(tournament.status, &stage, &viewer),
        _ => None,
    };
    Ok(BracketPlace {
        tournament,
        place,
        stage: number,
        next,
    })
}

/// Onde `user_id` (inscrito) terminou em cada torneio encerrado de
/// `finished`, na ordem, pela classificação final: só a chave das fases, sem
/// o veto, e duas idas ao banco para todos (`stage::brackets`). `None` no
/// torneio com confirmação por tempo vencida: a chave ainda pode mudar, e ele
/// vai por `place` (que aplica e trava).
pub async fn final_places(
    conn: &mut PgConnection,
    finished: &[TournamentId],
    user_id: i64,
) -> Result<Vec<Option<(Place, i16)>>, ApiError> {
    let now = Utc::now();
    Ok(stage::brackets(conn, finished)
        .await?
        .iter()
        .map(|c| {
            let due = c
                .stages
                .iter()
                .any(|s| due::confirmations_due(&s.games, now));
            let last = c.stages.last()?;
            let me = stage::participant_of(&c.teams, user_id);
            let place = match me {
                None => Place::Missed,
                Some(me) => in_standings(&chain::standings(c), &c.teams, me),
            };
            (!due).then_some((place, last.number))
        })
        .collect())
}

/// Como `place`, para outro inscrito, `user_id`, no torneio `slug` como
/// `member` o vê: sem a próxima partida, que é só de quem joga.
pub async fn place_of<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    slug: &str,
    user_id: i64,
) -> Result<BracketPlace, ApiError> {
    let (tournament, stage) = due::current(conn, member, slug, None).await?;
    let me = stage::participant_of(&stage.teams, user_id);
    let (place, number) = locate(conn, &tournament, &stage, me).await?;
    Ok(BracketPlace {
        tournament,
        place,
        stage: number,
        next: None,
    })
}

/// O lugar de `me` (um dos times que jogaram o torneio; `None`, o inscrito
/// sem seed) com a fase atual `stage`, e a fase de que ele fala. Só quem
/// ficou numa fase anterior pede as outras fases (uma ida ao banco).
async fn locate(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    stage: &Stage,
    me: Option<ParticipantId>,
) -> Result<(Place, i16), ApiError> {
    let Some(me) = me else {
        return Ok((Place::Missed, stage::FIRST_STAGE));
    };
    match in_current(stage.last, &stage.entrants, &stage.bracket, me) {
        Some(Place::Playing) if !stage.last && stage.bracket.is_finished() => {
            Ok((between(tournament, stage, me), stage.number + 1))
        }
        Some(place) => Ok((place, stage.number)),
        None => {
            let chain = stage::chain(conn, tournament, stage).await?;
            let standings = chain::standings(&chain);
            Ok((in_standings(&standings, &stage.teams, me), stage.number))
        }
    }
}

/// O lugar de `me` na fase atual (`last`: a última do torneio), dos times
/// `entrants`; `None` se ele não está nela. Antes da última fase, a colocação
/// dela não é a do torneio: quem está nela continua jogando até a fase
/// seguinte decidir.
fn in_current(
    last: bool,
    entrants: &[StoredTeam],
    bracket: &StageBracket,
    me: ParticipantId,
) -> Option<Place> {
    let entrant = entrants.iter().find(|e| e.id == me.0)?;
    let withdrawn = entrant.withdrawn;
    let tie = bracket
        .pending_ties()
        .iter()
        .find(|t| t.participants.contains(&me));
    if let Some(t) = tie.filter(|_| !withdrawn) {
        return Some(Place::TiePending(t.position, t.participants.len() as u16));
    }
    if !last {
        return Some(if withdrawn {
            Place::Withdrawn(None)
        } else {
            Place::Playing
        });
    }
    let standing = bracket
        .placements()
        .into_iter()
        .find(|p| p.participant == me)
        .map(|p| (p.rank, p.tied));
    Some(match standing {
        Some((1, _)) => Place::Champion,
        _ if withdrawn => Place::Withdrawn(standing),
        Some((rank, tied)) => Place::Eliminated(rank, tied),
        None => Place::Playing,
    })
}

/// O lugar de `me` pela classificação final `standings` (dos times `teams`):
/// quem ficou numa fase anterior, ou o torneio encerrado.
fn in_standings(standings: &[Placement], teams: &[StoredTeam], me: ParticipantId) -> Place {
    let standing = standings
        .iter()
        .find(|p| p.participant == me)
        .map(|p| (p.rank, p.tied));
    let withdrawn = teams.iter().any(|t| t.id == me.0 && t.withdrawn);
    match standing {
        Some((1, _)) => Place::Champion,
        _ if withdrawn => Place::Withdrawn(standing),
        Some((rank, tied)) => Place::Eliminated(rank, tied),
        None => Place::Playing,
    }
}

/// Entre as fases (a atual, `stage`, terminou e não é a última): `me`, que a
/// jogou e não desistiu, espera a seguinte, confirma presença ou é suplente,
/// por quem entraria agora (`chain::projection`).
fn between(tournament: &StoredTournament, stage: &Stage, me: ParticipantId) -> Place {
    let next = stage.number + 1;
    let Some((setup, progress)) = tournament.stage(next) else {
        return Place::Playing;
    };
    let Some(order) = chain::ranking(&stage.bracket) else {
        return Place::Playing;
    };
    let open = setup.check_in && progress.check_in_opened_at.is_some();
    let confirmed = stage
        .entrants
        .iter()
        .any(|t| t.id == me.0 && t.checked_in_for(next));
    if open && !confirmed {
        return Place::StageCheckIn;
    }
    let entry = chain::projection(&stage.entrants, &order, next, setup, progress);
    if entry.entrants.contains(&me) {
        Place::AwaitingStage
    } else {
        Place::Substitute
    }
}

/// Round robin: a série em que o adversário enviou um mapa, que tem prazo, ou que
/// tem uma contestação para ele aceitar, ou a sugerida (`progress`). `None` nos
/// outros formatos.
fn league_next<'a>(
    status: TournamentStatus,
    stage: &'a Stage,
    viewer: &Viewer,
) -> Option<&'a ec::Match> {
    let suggested = progress::suggestions(stage);
    let me = suggested.get(&viewer.participant?)?;
    let answer = stage.bracket.matches().iter().find(|m| {
        let side = view::viewer_side(stage, m, viewer);
        side.is_some()
            && matches!(
                tasks::match_task(status, stage, m, side, viewer),
                Some(ViewerTask::Respond | ViewerTask::AcceptContest)
            )
    });
    answer.or_else(|| stage.bracket.get(*me))
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
    let m = league_next(status, stage, viewer)
        .or_else(|| mine(true))
        .or_else(|| mine(false))?;
    let side = view::viewer_side(stage, m, viewer)?;
    let (oa, ob) = view::occupants(stage, m);
    let (opponent, slot) = match side {
        Side::A => (ob, m.b),
        Side::B => (oa, m.a),
    };
    Some(NextMatch {
        key: key::public(stage.number, m.id),
        state: match m.state {
            ec::MatchState::Ready { .. } => MatchState::Ready,
            _ => MatchState::Pending,
        },
        single: matches!(
            stage.format,
            ef::StageFormat::Elimination(c) if c.kind == EliminationKind::Single
        ),
        opponent: match opponent {
            Occupant::Participant(p) => stage.entrant(p).map(StoredTeam::view),
            _ => None,
        },
        opponent_from: match (opponent, slot) {
            (Occupant::Unknown, Slot::WinnerOf(id)) => Some(SideSource {
                outcome: SourceOutcome::Winner,
                match_key: key::public(stage.number, id),
            }),
            (Occupant::Unknown, Slot::LoserOf(id)) => Some(SideSource {
                outcome: SourceOutcome::Loser,
                match_key: key::public(stage.number, id),
            }),
            _ => None,
        },
        task: tasks::match_task(status, stage, m, Some(side), viewer),
        veto: veto::status(stage, m),
        next_map: veto::next_map(status, stage, m, &stage.games_of(m.id)),
        opponent_playing: progress::opponent_playing(
            stage,
            m,
            Some(side),
            &progress::playing(stage),
        ),
    })
}
