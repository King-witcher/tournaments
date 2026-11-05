//! Os torneios de um membro na comunidade, para o perfil dele: os da lista
//! que quem vê vê (`tournaments::list_in`, a mesma de
//! `GET /tenants/{slug}/tournaments`) em que o membro está inscrito, com a
//! situação, a colocação e o time dele (`my_tournaments::entries`, a mesma
//! entrada de `/me/tournaments`: o time e a colocação nos encerrados lidos
//! de todos os torneios de uma vez; a fase inteira, só nos em andamento).

use std::{cmp::Reverse, collections::HashMap};

use chrono::{DateTime, Utc};
use sqlx::PgConnection;

use super::types::MemberTournament;
use crate::{
    my_tournaments::{self, Registered},
    participants,
    shared::ApiError,
    tenants::{Role, TenantId, TenantMember},
    tournaments::{self, TournamentSummary},
};

/// Os torneios em que `user_id` está inscrito, como `viewer` os vê, do mais
/// recente ao mais antigo (`recency`). Nos que têm chave, aplica antes as
/// confirmações por tempo vencidas, como `/me/tournaments`.
pub async fn of<R>(
    conn: &mut PgConnection,
    viewer: &TenantMember<R>,
    user_id: i64,
) -> Result<Vec<MemberTournament>, ApiError> {
    let drafts: &[TenantId] = if viewer.role >= Role::Organizer {
        &[viewer.tenant]
    } else {
        &[]
    };
    let visible = tournaments::list_in(conn, &[viewer.tenant], drafts, viewer.user.id).await?;
    let checked_in: HashMap<String, bool> =
        participants::registrations_of(conn, viewer.tenant, user_id)
            .await?
            .into_iter()
            .map(|r| (r.tournament, r.checked_in))
            .collect();
    let mut found: Vec<Registered<'_, R>> = visible
        .into_iter()
        .filter_map(|l| {
            let &checked_in = checked_in.get(&l.summary.slug)?;
            Some(Registered {
                member: viewer,
                id: l.id,
                tournament: l.summary,
                checked_in,
                found: None,
            })
        })
        .collect();
    found.sort_by_key(|r| Reverse(recency(&r.tournament)));
    Ok(my_tournaments::entries(conn, user_id, found)
        .await?
        .into_iter()
        .map(|e| MemberTournament {
            tournament: e.tournament,
            situation: e.situation,
            placement: e.placement,
            team: e.team,
        })
        .collect())
}

/// Ordem crescente de recência: sem data de início (só rascunho) é o mais
/// recente; depois o início e, no empate, a criação.
fn recency(t: &TournamentSummary) -> (bool, Option<DateTime<Utc>>, DateTime<Utc>) {
    (t.starts_at.is_none(), t.starts_at, t.created_at)
}
