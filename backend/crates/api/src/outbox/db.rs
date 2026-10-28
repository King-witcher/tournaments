//! SQL do outbox: gravar na transação de quem muda e, no despachante, pegar
//! um lote e apagar o que saiu.

use serde_json::Value;
use sqlx::{PgConnection, types::Json};

use super::{event::Event, tracking};

/// Grava `event` na transação de `conn`: só sai depois do commit, e o mesmo
/// evento repetido na transação entra uma vez. Evento sem destinatário não é
/// gravado.
pub async fn emit(conn: &mut PgConnection, event: Event) -> Result<(), sqlx::Error> {
    if event.is_empty() {
        return Ok(());
    }
    sqlx::query!(
        r#"insert into outbox (event) values ($1)
           on conflict (xact, (md5(event::text))) do nothing"#,
        Json(&event) as _
    )
    .execute(conn)
    .await?;
    tracking::mark();
    Ok(())
}

/// Um evento lido do outbox; `None` se a forma não é a que a API conhece.
pub struct Stored {
    pub id: i64,
    pub event: Option<Event>,
}

/// Até `limit` eventos confirmados, em ordem, travados até o fim da transação
/// (outra instância pula os travados).
pub async fn claim(conn: &mut PgConnection, limit: i64) -> Result<Vec<Stored>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"select id, event as "event: Json<Value>"
           from outbox
           order by id
           limit $1
           for update skip locked"#,
        limit
    )
    .fetch_all(conn)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| {
            let event = serde_json::from_value(r.event.0)
                .inspect_err(|e| tracing::error!(id = r.id, %e, "evento do outbox ilegível"))
                .ok();
            Stored { id: r.id, event }
        })
        .collect())
}

/// Apaga os eventos que já saíram.
pub async fn remove(conn: &mut PgConnection, ids: &[i64]) -> Result<(), sqlx::Error> {
    sqlx::query!("delete from outbox where id = any($1)", ids)
        .execute(conn)
        .await?;
    Ok(())
}
