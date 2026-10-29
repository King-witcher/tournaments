//! O que a chave lê dos prints (a tabela é do domínio `prints`): se um mapa
//! tem print anexado, para o print obrigatório no envio.

use sqlx::PgConnection;

use super::MatchRef;

/// O mapa `number` da partida tem ao menos um print anexado (o upload
/// conferido; o só pedido não conta).
pub async fn has_print(
    conn: &mut PgConnection,
    at: &MatchRef<'_>,
    number: i16,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select exists(select 1 from match_prints
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
