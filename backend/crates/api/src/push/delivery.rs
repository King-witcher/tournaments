//! O envio de um aviso por Web Push, que o despachante do tempo real chama:
//! uma requisição por inscrição das pessoas do aviso. O corpo é o `Notice`
//! em JSON (`{code, params, path}`), que o service worker traduz no idioma da
//! inscrição.

use sqlx::PgConnection;

use super::{
    db,
    sender::{Sent, WebPush},
};
use crate::outbox::Notice;

/// Manda `notice` para as inscrições de `users`. A inscrição que o serviço diz
/// não existir mais (404, 410) sai; erro de rede ou outra resposta fica no log
/// e não segura os outros envios. Roda na transação do despachante.
pub async fn deliver(
    conn: &mut PgConnection,
    push: &dyn WebPush,
    users: &[i64],
    notice: &Notice,
) -> Result<(), sqlx::Error> {
    let targets = db::of_users(conn, users).await?;
    if targets.is_empty() {
        return Ok(());
    }
    let payload = serde_json::to_vec(notice).expect("o aviso vira JSON");
    let (mut delivered, mut gone) = (Vec::new(), Vec::new());
    for target in &targets {
        match push.send(target, &payload).await {
            Ok(Sent::Delivered) => delivered.push(target.id),
            Ok(Sent::Gone) => gone.push(target.id),
            Err(e) => tracing::warn!(id = target.id, error = %e, "Web Push falhou"),
        }
    }
    if !gone.is_empty() {
        db::remove(conn, &gone).await?;
    }
    if !delivered.is_empty() {
        db::mark_used(conn, &delivered).await?;
    }
    Ok(())
}
