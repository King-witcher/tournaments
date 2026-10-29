//! Screenshot obrigatório: com o torneio exigindo e o bucket configurado, o
//! mapa que o jogador envia precisa ter ao menos um screenshot anexado (os
//! screenshots são do domínio `screenshots`; aqui só a regra do envio). Lançar,
//! corrigir e resolver, pelo organizador, não exigem.

use sqlx::PgConnection;

use super::db::{self, MatchRef};
use crate::{shared::ApiError, tournaments::StoredTournament};

/// 409 `match.screenshot_required` se o envio do mapa `number` precisava de
/// screenshot e não há nenhum anexado. `enabled`: o bucket está configurado
/// (sem ele, o screenshot obrigatório não vale).
pub async fn require(
    conn: &mut PgConnection,
    enabled: bool,
    tournament: &StoredTournament,
    at: &MatchRef<'_>,
    number: i16,
) -> Result<(), ApiError> {
    if !enabled || !tournament.screenshots_required || db::has_screenshot(conn, at, number).await? {
        return Ok(());
    }
    Err(ApiError::conflict("match.screenshot_required"))
}
