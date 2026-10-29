//! O que a chave lê dos screenshots (a tabela é do domínio `screenshots`): se
//! um mapa tem screenshot anexado, para o screenshot obrigatório no envio.

use sqlx::PgConnection;

use super::MatchRef;

/// O mapa `number` da partida tem ao menos um screenshot anexado (o upload
/// conferido; o só pedido não conta).
pub async fn has_screenshot(
    conn: &mut PgConnection,
    at: &MatchRef<'_>,
    number: i16,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select exists(select 1 from match_screenshots
                         where tenant_id = $1 and tournament_id = $2 and stage = $3
                           and match_key = $4 and number = $5 and attached_at is not null)
           as "exists!""#,
        at.tenant.get(),
        at.tournament.get(),
        at.stage,
        at.key,
        number
    )
    .fetch_one(conn)
    .await
}
