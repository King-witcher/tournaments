//! Os torneios de um membro na comunidade, para o perfil dele: os da lista
//! que quem vê vê (`tournaments::list`, a mesma de
//! `GET /tenants/{slug}/tournaments`) em que o membro está inscrito, com a
//! situação e a colocação dele (`my_tournaments::Situation`, pelo lugar na
//! chave de `matches::place_of`).

use std::{cmp::Reverse, collections::HashMap};

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::types::MemberTournament;
use crate::{
    matches,
    my_tournaments::{self, Situation},
    participants,
    shared::ApiError,
    tenants::{Role, TenantMember},
    tournaments::{self, TournamentMode, TournamentSummary},
};

/// Os torneios em que `user_id` está inscrito, como `viewer` os vê, do mais
/// recente ao mais antigo (`recency`). Nos que têm chave, aplica antes as
/// confirmações por tempo vencidas, como `/me/tournaments`.
pub async fn of<R>(
    db: &PgPool,
    viewer: &TenantMember<R>,
    user_id: i64,
) -> Result<Vec<MemberTournament>, ApiError> {
    let see_all = viewer.role >= Role::Organizer;
    let visible = tournaments::list(db, viewer.tenant, viewer.user.id, see_all).await?;
    let mut conn = db.acquire().await?;
    let checked_in: HashMap<String, bool> =
        participants::registrations_of(&mut conn, viewer.tenant, user_id)
            .await?
            .into_iter()
            .map(|r| (r.tournament, r.checked_in))
            .collect();
    let mut found: Vec<(TournamentSummary, bool)> = visible
        .into_iter()
        .filter_map(|t| checked_in.get(&t.slug).map(|&c| (t, c)))
        .collect();
    found.sort_by_key(|(t, _)| Reverse(recency(t)));

    let mut out = Vec::with_capacity(found.len());
    for (mut tournament, checked_in) in found {
        let before = Situation::before_bracket(tournament.status, checked_in);
        let (situation, placement) = match before {
            Some(situation) => (situation, None),
            None => {
                let place = matches::place_of(&mut conn, viewer, &tournament.slug, user_id).await?;
                tournament.status = place.tournament.status;
                let teams = tournament.mode == TournamentMode::Tdm;
                Situation::in_bracket(place.place, teams)
            }
        };
        let team = my_tournaments::team_of(&mut conn, viewer.tenant, &tournament, user_id).await?;
        out.push(MemberTournament {
            tournament,
            situation,
            placement,
            team,
        });
    }
    Ok(out)
}

/// Ordem crescente de recência: sem data de início (só rascunho) é o mais
/// recente; depois o início e, no empate, a criação.
fn recency(t: &TournamentSummary) -> (bool, Option<DateTime<Utc>>, DateTime<Utc>) {
    (t.starts_at.is_none(), t.starts_at, t.created_at)
}
