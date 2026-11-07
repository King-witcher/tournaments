//! O aviso de partida contestada à organização, que o agendador manda depois de
//! `contest::STAFF_NOTICE_SECONDS` se ela continua contestada: o capitão adversário,
//! avisado na hora, pode ter resolvido antes (`accept`), e aí ninguém é incomodado.
//! A organização vê a partida e pode resolver desde o começo; só o aviso espera.

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};

use super::{changes, contest, db, key, stage};
use crate::{
    shared::ApiError,
    tournaments::{self, StoredTournament},
};

/// Manda os avisos vencidos em `now`, cada torneio na própria transação e com o
/// torneio travado. Devolve quantos mandou; o erro de um torneio fica no log e não
/// para os outros.
pub async fn notify_contested(db: &PgPool, now: DateTime<Utc>) -> Result<u64, ApiError> {
    let mut sent = 0;
    for (tenant, slug) in db::contest_notices_due(db, now).await? {
        let notify = async {
            let mut tx = db.begin().await?;
            let Some(t) = tournaments::find_for_update(&mut tx, tenant, &slug, None).await? else {
                return Ok(0);
            };
            let n = notify_tournament(&mut tx, &t, now).await?;
            tx.commit().await?;
            Ok::<_, ApiError>(n)
        };
        match notify.await {
            Ok(n) => sent += n,
            Err(e) => tracing::error!(slug, code = e.code, "aviso de contestação falhou"),
        }
    }
    Ok(sent)
}

/// Quem já foi avisado na hora não recebe de novo: o capitão adversário que
/// também organiza. Quem contestou não avisa a si mesmo.
async fn notify_tournament(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    now: DateTime<Utc>,
) -> Result<u64, ApiError> {
    let due = db::take_contest_notices(conn, tournament.id, now).await?;
    // A partida pode ser de uma fase anterior (contestada antes de a seguinte
    // começar): cada fase dos avisos é lida uma vez.
    let mut loaded: Option<stage::Stage> = None;
    for notice in &due {
        if loaded.as_ref().is_none_or(|s| s.number != notice.stage) {
            loaded = Some(stage::load(conn, tournament, notice.stage).await?);
        }
        let stage = loaded.as_ref().ok_or_else(ApiError::internal)?;
        let captain = stage
            .games
            .iter()
            .find(|g| g.id == notice.game)
            .and_then(|g| contest::accepter(stage, g));
        let except = [notice.contested_by, captain];
        let player = notice.player.as_deref().unwrap_or_default();
        changes::game_contested(
            conn,
            tournament,
            &key::prefixed(notice.stage, &notice.match_key),
            notice.number,
            player,
            except,
        )
        .await?;
    }
    Ok(due.len() as u64)
}
