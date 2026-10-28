//! O que a comunidade emite no outbox: a mudança (membros, papéis, apelidos,
//! convites, lista de torneios) e o acesso perdido de quem saiu.

use sqlx::PgConnection;

use super::{TenantId, db};
use crate::outbox::{self, Event};

/// A comunidade mudou: as abas dela recarregam.
pub async fn changed(conn: &mut PgConnection, tenant: TenantId) -> Result<(), sqlx::Error> {
    let tenant = db::slug_of(conn, tenant).await?;
    outbox::emit(conn, Event::Tenant { tenant }).await
}

/// `user` saiu ou foi removido: as abas dele perdem as assinaturas da
/// comunidade e recarregam, e as dos outros membros veem a lista nova.
pub async fn access_lost(
    conn: &mut PgConnection,
    tenant: TenantId,
    user: i64,
) -> Result<(), sqlx::Error> {
    let tenant = db::slug_of(conn, tenant).await?;
    outbox::emit(
        conn,
        Event::Tenant {
            tenant: tenant.clone(),
        },
    )
    .await?;
    outbox::emit(conn, Event::AccessLost { user, tenant }).await?;
    outbox::emit(conn, Event::me([user])).await
}
