use axum::{Json, extract::State};

use super::types::{MyTournament, Situation};
use crate::{
    AppState,
    auth::CurrentUser,
    matches,
    shared::{ApiError, ErrorBody},
    tenants::{Role, TenantMember},
    tournaments::{self, TournamentStatus, TournamentSummary},
};

/// Encerrados que entram na lista: os de início mais recente. Cada um refaz
/// a chave para dizer a colocação.
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
        (status = OK, body = Vec<MyTournament>, description = "Torneios em que quem vê está inscrito, de todas as comunidades de que participa: primeiro os que não encerraram (em andamento, check-in, inscrição, rascunho; dentro de cada um, pelo início), depois os 5 encerrados de início mais recente. As confirmações por tempo vencidas são aplicadas antes"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
    )
)]
pub async fn my_tournaments(
    State(state): State<AppState>,
    user: CurrentUser,
) -> Result<Json<Vec<MyTournament>>, ApiError> {
    let members = TenantMember::all_of(&state.db, &user).await?;
    let mut found: Vec<(&TenantMember, TournamentSummary)> = Vec::new();
    for member in &members {
        let see_all = member.role >= Role::Organizer;
        let list = tournaments::list(&state.db, member.tenant, user.id, see_all).await?;
        found.extend(
            list.into_iter()
                .filter(|t| t.registered)
                .map(|t| (member, t)),
        );
    }
    let (mut finished, mut active): (Vec<_>, Vec<_>) = found
        .into_iter()
        .partition(|(_, t)| t.status == TournamentStatus::Finished);
    let rank = |s: TournamentStatus| ACTIVE_ORDER.iter().position(|o| *o == s);
    active.sort_by(|(_, x), (_, y)| {
        rank(x.status)
            .cmp(&rank(y.status))
            .then(x.starts_at.cmp(&y.starts_at))
    });
    finished.sort_by_key(|(_, t)| std::cmp::Reverse(t.starts_at));
    finished.truncate(RECENT_FINISHED);

    let mut conn = state.db.acquire().await?;
    let mut out = Vec::with_capacity(active.len() + finished.len());
    for (member, mut tournament) in active.into_iter().chain(finished) {
        let before = Situation::before_bracket(tournament.status, tournament.checked_in);
        let (situation, placement, next_match) = match before {
            Some(situation) => (situation, None, None),
            None => {
                let place = matches::place(&mut conn, member, &tournament.slug).await?;
                tournament.status = place.tournament.status;
                let (situation, placement) = Situation::in_bracket(place.place);
                (situation, placement, place.next)
            }
        };
        out.push(MyTournament {
            tenant_slug: member.slug.clone(),
            tenant_name: member.name.clone(),
            tournament,
            situation,
            placement,
            next_match,
        });
    }
    Ok(Json(out))
}
