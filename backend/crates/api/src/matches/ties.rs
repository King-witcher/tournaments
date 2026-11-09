//! O empate do fim de uma fase de tabela com a organização decidindo
//! (`final_tie = organizer`; a regra é do `engine`, `formats::table::ties`).
//! A fase espera: com algum empate pendente, ela não termina, mesmo com todas
//! as séries decididas. A organização decide um grupo por vez, com a ordem
//! completa e um motivo opcional (`decide_tie`); usar a ordem da chave também
//! grava uma decisão. Cada decisão vale pelo conjunto de times: a correção que
//! muda quem está empatado a deixa sem efeito, e o grupo volta a ser pendente.
//!
//! Depois de toda mudança (`settle`), `sync` acerta a marca de espera
//! (`stage_pending_ties`): quando a fase passa a esperar, avisa a organização
//! uma vez (toast, Web Push e Discord, como a partida contestada) e a home
//! dela recarrega; quando deixa de esperar, só a home. Ao resolver o último
//! pendente, a fase termina pelo `settle`, como no encerramento normal.

use axum::{Json, extract::State};
use engine::{
    core::ParticipantId,
    formats::{
        StageBracket,
        table::{PendingTie, TieOrder},
    },
};
use sqlx::PgConnection;

use super::{
    changes::Snapshot,
    db::{self, TieRow},
    due, settle,
    stage::Stage,
    types::{TieDecision, TournamentBracket},
    view::{self, Viewer},
};
use crate::{
    AppState,
    outbox::{self, Event, Notice, NoticeKind},
    shared::{ApiError, ErrorBody, JsonBody, PathParams, text::clean_text},
    tenants::{self, Organizer, TenantMember},
    tournaments::{self, StoredTournament, TournamentPath},
};

/// O motivo da decisão, opcional.
const MAX_REASON: usize = 280;

/// Uma decisão gravada: os times na ordem decidida, do melhor ao pior.
pub struct StoredTie {
    pub teams: Vec<ParticipantId>,
    pub reason: Option<String>,
}

impl StoredTie {
    /// Como o `engine` recebe.
    pub fn order(&self) -> TieOrder {
        TieOrder {
            participants: self.teams.clone(),
        }
    }
}

/// As decisões como vieram de `stage_rows`, na ordem em que foram gravadas.
pub fn stored(rows: Vec<TieRow>) -> Vec<StoredTie> {
    rows.into_iter()
        .map(|r| StoredTie {
            teams: r.teams.into_iter().map(ParticipantId).collect(),
            reason: r.reason,
        })
        .collect()
}

/// A decisão que vale para o grupo `group` (times de uma ordem que a
/// organização decidiu): a última gravada com o mesmo conjunto, como no
/// `engine`.
pub fn decision_for<'a>(stage: &'a Stage, group: &[ParticipantId]) -> Option<&'a StoredTie> {
    stage.ties.iter().rev().find(|d| d.order().is_for(group))
}

/// Depois de `settle`: marca a fase que passou a esperar a organização (e a
/// avisa, uma vez) ou desmarca a que deixou de esperar.
pub async fn sync(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    stage: &Stage,
) -> Result<(), ApiError> {
    let waiting = !stage.bracket.pending_ties().is_empty();
    let marked = stage.ties_pending_since.is_some();
    if waiting == marked {
        return Ok(());
    }
    if waiting {
        let at = (tournament.tenant, tournament.id, stage.number);
        if db::mark_pending(conn, at).await? {
            notify(conn, tournament).await?;
        }
    } else {
        db::clear_pending(conn, tournament.id, stage.number).await?;
    }
    tenants::organizing_changed(conn, tournament.tenant).await?;
    Ok(())
}

/// `tournament.tie_pending` para organizers e owner, com a aba Classificação.
async fn notify(conn: &mut PgConnection, tournament: &StoredTournament) -> Result<(), ApiError> {
    let audience = tournament.address();
    let staff = tenants::staff(conn, tournament.tenant).await?;
    let notice = Notice {
        kind: NoticeKind::TiePending {
            tournament: audience.name.clone(),
        },
        path: format!("{}/standings", audience.path()),
    };
    outbox::emit(conn, Event::notice(staff, notice)).await?;
    Ok(())
}

/// O grupo pendente com exatamente os times de `order`. 409
/// `tie.not_pending` quando nenhum bate (a fase não espera decisão, ou o grupo
/// mudou).
fn pending_group<'a>(
    bracket: &'a StageBracket,
    order: &[ParticipantId],
) -> Result<&'a PendingTie, ApiError> {
    bracket
        .pending_ties()
        .iter()
        .find(|t| t.is_order(order))
        .ok_or(ApiError::conflict("tie.not_pending"))
}

/// Grava a decisão e devolve a chave como ficou, para quem decidiu: o `settle`
/// termina a fase se não sobrou pendente, e a página do torneio e "Seus
/// torneios" dos inscritos recarregam.
async fn write(
    conn: &mut PgConnection,
    member: &TenantMember<Organizer>,
    mut tournament: StoredTournament,
    stage: &Stage,
    teams: &[ParticipantId],
    reason: Option<&str>,
) -> Result<TournamentBracket, ApiError> {
    let at = (tournament.tenant, tournament.id, stage.number);
    db::insert_decision(conn, at, teams, member.user.id, reason).await?;
    let before = Snapshot::of(stage);
    let (stage, status) = settle::settle(conn, &tournament, Some(member.user.id), &before).await?;
    tournaments::changed(conn, tournament.id).await?;
    tournament.status = status;
    let viewer = Viewer::new(member, &stage);
    let chain = super::stage::chain(conn, &tournament, &stage).await?;
    view::bracket(&tournament, &stage, &viewer, &chain)
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/ties",
    tag = "matches",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    request_body = TieDecision,
    responses(
        (status = OK, body = TournamentBracket, description = "A ordem de um empate pendente do fim da fase (`pending_ties` da chave), decidida pela organização, com o motivo opcional; usar a ordem da chave também é mandar a ordem dela. Sem outro pendente, o torneio encerra"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tie.not_pending (os times não são exatamente os de um empate pendente agora)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "tie.reason_invalid {max}, request.invalid_body"),
    )
)]
pub async fn decide_tie(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
    JsonBody(body): JsonBody<TieDecision>,
) -> Result<Json<TournamentBracket>, ApiError> {
    let reason = body
        .reason
        .as_deref()
        .map(|r| {
            clean_text(r, MAX_REASON, "tie.reason_invalid")
                .map_err(|e| e.with_params([("max", MAX_REASON as i64)]))
        })
        .transpose()?;
    let order: Vec<ParticipantId> = body.teams.into_iter().map(ParticipantId).collect();
    let mut tx = state.db.begin().await?;
    let (tournament, stage) = due::lock_stage(&mut tx, &member, &path.tournament, None).await?;
    pending_group(&stage.bracket, &order)?;
    let bracket = write(
        &mut tx,
        &member,
        tournament,
        &stage,
        &order,
        reason.as_deref(),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(bracket))
}
