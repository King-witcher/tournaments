//! O que os screenshots emitem no outbox: a partida mudou (a página dela
//! recarrega e mostra o screenshot novo, ou some o removido). Sem aviso para
//! pessoas.

use sqlx::PgConnection;

use crate::{
    matches::MatchGames,
    outbox::{self, Event},
    tenants::TenantMember,
};

pub async fn changed<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    m: &MatchGames,
) -> Result<(), sqlx::Error> {
    let event = Event::Tournament {
        tenant: member.slug.clone(),
        tournament: m.tournament.slug.clone(),
        match_key: Some(m.key.clone()),
    };
    outbox::emit(conn, event).await
}
