//! O que a comunidade emite no outbox: a mudança (membros, papéis, apelidos,
//! convites, lista de torneios), o acesso perdido de quem saiu e a mudança no
//! que espera a organização dela (a home de organizers e owner).

use sqlx::PgConnection;

use super::{TenantId, db};
use crate::outbox::{self, Event};

/// A comunidade mudou: as abas dela recarregam.
pub async fn changed(conn: &mut PgConnection, tenant: TenantId) -> Result<(), sqlx::Error> {
    let tenant = db::slug_of(conn, tenant).await?;
    outbox::emit(conn, Event::Tenant { tenant }).await
}

/// O que espera a organização da comunidade mudou (mapa contestado, hora de
/// iniciar; `GET /me/organizing`): a home dos organizers e do owner
/// recarrega, pelo canal deles (`me`).
pub async fn organizing_changed(
    conn: &mut PgConnection,
    tenant: TenantId,
) -> Result<(), sqlx::Error> {
    let staff = db::staff(conn, tenant).await?;
    outbox::emit(conn, Event::me(staff)).await
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
