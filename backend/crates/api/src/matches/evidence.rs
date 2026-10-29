//! Print obrigatório: com o torneio exigindo e o bucket configurado, o mapa
//! que o jogador envia precisa ter ao menos um print anexado (os prints são do
//! domínio `prints`; aqui só a regra do envio). Lançar, corrigir e resolver,
//! pelo organizador, não exigem.

use sqlx::PgConnection;

use super::db::{self, MatchRef};
use crate::{shared::ApiError, tournaments::StoredTournament};

/// 409 `match.print_required` se o envio do mapa `number` precisava de print e
/// não há nenhum anexado. `enabled`: o bucket está configurado (sem ele, o
/// print obrigatório não vale).
pub async fn require(
    conn: &mut PgConnection,
    enabled: bool,
    tournament: &StoredTournament,
    at: &MatchRef<'_>,
    number: i16,
) -> Result<(), ApiError> {
    if !enabled || !tournament.prints_required || db::has_print(conn, at, number).await? {
        return Ok(());
    }
    Err(ApiError::conflict("match.print_required"))
}
