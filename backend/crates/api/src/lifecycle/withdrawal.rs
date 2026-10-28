//! Quem deixa os torneios no meio do caminho: o membro que sai da comunidade
//! (ou é removido) sai das inscrições dos torneios que não começaram; nos
//! iniciados, a inscrição fica como histórico.

use sqlx::PgConnection;

use crate::{
    participants,
    shared::ApiError,
    tenants::TenantId,
    tournaments::{self},
};

/// `user_id` deixa o tenant: sai das inscrições dos torneios que não
/// começaram. Trava, na ordem dos ids, todo torneio que a saída pode mudar
/// (também os que não encerraram e em que ele não está, para uma inscrição
/// dele no mesmo instante esperar e ver que ele saiu).
pub async fn leave_tournaments(
    conn: &mut PgConnection,
    tenant: TenantId,
    user_id: i64,
) -> Result<(), ApiError> {
    for slug in tournaments::slugs_for_member(conn, tenant, user_id).await? {
        let Some(t) = tournaments::find_for_update(conn, tenant, &slug, Some(user_id)).await?
        else {
            continue;
        };
        if t.registered && t.status.participants_editable() {
            participants::unregister(conn, t.id, user_id).await?;
        }
    }
    Ok(())
}
