use axum::{Json, extract::State};

use super::{
    entry::{Registered, entries},
    types::{MyTournament, MyTournaments, OpenTournament},
};
use crate::{
    AppState, participants,
    shared::{ApiError, ErrorBody, PathParams, public_url},
    tenants::{Memberships, Role, TenantId, TenantMember},
    tournaments::{
        self, Listed, TournamentPath, TournamentStatus, TournamentSummary, find_visible,
    },
};

/// Encerrados que entram na lista: os de início mais recente.
const RECENT_FINISHED: usize = 5;

/// Ordem dos que não encerraram: em andamento primeiro, onde há partida
/// esperando.
const ACTIVE_ORDER: [TournamentStatus; 4] = [
    TournamentStatus::InProgress,
    TournamentStatus::CheckIn,
    TournamentStatus::Registration,
    TournamentStatus::Draft,
];

#[utoipa::path(
    get,
    path = "/me/tournaments",
    tag = "tournaments",
    responses(
        (status = OK, body = MyTournaments, description = "Os torneios de quem vê, de todas as comunidades de que participa: em que está inscrito (`registered`, com a situação e a próxima partida) e em que pode se inscrever agora pelo botão (`open`). As confirmações por tempo vencidas são aplicadas antes"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
    )
)]
pub async fn my_tournaments(
    State(state): State<AppState>,
    me: Memberships,
) -> Result<Json<MyTournaments>, ApiError> {
    if me.members.is_empty() {
        return Ok(Json(MyTournaments {
            registered: Vec::new(),
            open: Vec::new(),
        }));
    }
    let tenants: Vec<TenantId> = me.members.iter().map(|m| m.tenant).collect();
    let drafts: Vec<TenantId> = me
        .members
        .iter()
        .filter(|m| m.role >= Role::Organizer)
        .map(|m| m.tenant)
        .collect();
    // Uma conexão para a lista e as entradas.
    let mut conn = state.db.acquire().await?;
    let mut listed = tournaments::list_in(&mut conn, &tenants, &drafts, me.user.id).await?;
    // Comunidade a comunidade, na ordem em que entrou, e o mais novo primeiro
    // em cada uma: a ordem dos empates abaixo.
    let position = |tenant: TenantId| tenants.iter().position(|t| *t == tenant);
    listed.sort_by_key(|l| position(l.tenant));

    let mut found: Vec<(&TenantMember, Listed)> = Vec::new();
    let mut open: Vec<(&TenantMember, TournamentSummary)> = Vec::new();
    for l in listed {
        let member = position(l.tenant)
            .map(|i| &me.members[i])
            .ok_or_else(ApiError::internal)?;
        let t = &l.summary;
        if t.registered {
            found.push((member, l));
        } else if participants::open_to_members(
            t.status,
            t.admission,
            t.participants,
            t.max_participants,
        ) {
            open.push((member, l.summary));
        }
    }
    open.sort_by_key(|(_, t)| nulls_last(t.starts_at));
    let open = open
        .into_iter()
        .map(|(member, tournament)| OpenTournament {
            tenant_slug: member.slug.clone(),
            tenant_name: member.name.clone(),
            tenant_logo_url: member.logo_key.as_deref().and_then(public_url),
            tenant_cover_url: member.cover_key.as_deref().and_then(public_url),
            tournament,
        })
        .collect();

    let (mut finished, mut active): (Vec<_>, Vec<_>) = found
        .into_iter()
        .partition(|(_, l)| l.summary.status == TournamentStatus::Finished);
    let rank = |s: TournamentStatus| ACTIVE_ORDER.iter().position(|o| *o == s);
    active.sort_by(|(_, x), (_, y)| {
        rank(x.summary.status)
            .cmp(&rank(y.summary.status))
            .then(nulls_last(x.summary.starts_at).cmp(&nulls_last(y.summary.starts_at)))
    });
    finished.sort_by_key(|(_, l)| std::cmp::Reverse(l.summary.starts_at));
    finished.truncate(RECENT_FINISHED);

    let registered = active
        .into_iter()
        .chain(finished)
        .map(|(member, l)| Registered {
            member,
            id: l.id,
            checked_in: l.summary.checked_in,
            tournament: l.summary,
            found: None,
        })
        .collect();
    let registered = entries(&mut conn, me.user.id, registered).await?;
    Ok(Json(MyTournaments { registered, open }))
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}/tournaments/{tournament}/mine",
    tag = "tournaments",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = OK, body = MyTournament, description = "A entrada de quem vê neste torneio, a mesma que `/me/tournaments` traria em `registered` (também nos encerrados que a lista já não traz): a situação, a colocação, a próxima partida e o time. As confirmações por tempo vencidas são aplicadas antes"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, participant.not_found (quem vê não está inscrito)"),
    )
)]
pub async fn my_tournament(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<Json<MyTournament>, ApiError> {
    let mut conn = state.db.acquire().await?;
    let found = find_visible(&mut conn, &member, &path.tournament).await?;
    if !found.registered {
        return Err(ApiError::not_found("participant.not_found"));
    }
    let registered = Registered {
        member: &member,
        id: found.id,
        checked_in: found.checked_in,
        tournament: TournamentSummary::from(&found),
        found: Some(found),
    };
    let mut entry = entries(&mut conn, member.user.id, vec![registered]).await?;
    entry.pop().map(Json).ok_or_else(ApiError::internal)
}

/// Chave de ordenação crescente que põe os sem data depois dos com data.
fn nulls_last(
    starts_at: Option<chrono::DateTime<chrono::Utc>>,
) -> (bool, Option<chrono::DateTime<chrono::Utc>>) {
    (starts_at.is_none(), starts_at)
}
