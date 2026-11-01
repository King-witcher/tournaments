//! SQL do outbox: gravar na transação de quem muda e, no despachante, pegar
//! um lote e apagar o que saiu.

use serde_json::Value;
use sqlx::{PgConnection, types::Json};

use super::{event::Event, tracking};

/// Grava `event` na transação de `conn`: só sai depois do commit, e o mesmo
/// evento repetido na transação entra uma vez. Evento sem destinatário não é
/// gravado. Vai com a aba de onde veio a requisição (`track`), se ela disse.
pub async fn emit(conn: &mut PgConnection, event: Event) -> Result<(), sqlx::Error> {
    if event.is_empty() {
        return Ok(());
    }
    sqlx::query!(
        r#"insert into outbox (event, origin) values ($1, $2)
           on conflict (xact, (md5(event::text))) do nothing"#,
        Json(&event) as _,
        tracking::origin()
    )
    .execute(conn)
    .await?;
    tracking::mark();
    Ok(())
}

/// Como `emit`, com vários eventos numa ida ao banco, na ordem.
pub async fn emit_all(
    conn: &mut PgConnection,
    events: impl IntoIterator<Item = Event>,
) -> Result<(), sqlx::Error> {
    let events: Vec<Value> = events
        .into_iter()
        .filter(|e| !e.is_empty())
        .map(|e| serde_json::to_value(e).expect("o evento vira JSON"))
        .collect();
    if events.is_empty() {
        return Ok(());
    }
    sqlx::query!(
        r#"insert into outbox (event, origin)
           select e.event, $2 from unnest($1::jsonb[]) with ordinality as e(event, n)
           order by e.n
           on conflict (xact, (md5(event::text))) do nothing"#,
        &events,
        tracking::origin()
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
    /// A conexão do WebSocket da aba que pediu a mudança, se ela disse.
    pub origin: Option<String>,
}

/// Até `limit` eventos confirmados, em ordem, travados até o fim da transação
/// (outra instância pula os travados).
pub async fn claim(conn: &mut PgConnection, limit: i64) -> Result<Vec<Stored>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"select id, event as "event: Json<Value>", origin
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
            Stored {
                id: r.id,
                event,
                origin: r.origin,
            }
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
