//! Testes do tempo real: os eventos que cada ação grava no outbox
//! (`events`), os avisos e seus destinatários (`notices`), o despachante
//! (`dispatcher`), o agendador (`scheduler`) e o WebSocket (`socket`). As
//! tarefas em segundo plano não sobem nos testes: eles chamam `dispatch_once`
//! e `tick` direto.

mod dispatcher;
mod events;
mod notices;
mod scheduler;
mod socket;

use serde_json::{Value, json};
use sqlx::PgPool;

/// Os eventos que esperam o despachante, na ordem.
pub async fn outbox(db: &PgPool) -> Vec<Value> {
    sqlx::query_scalar("select event from outbox order by id")
        .fetch_all(db)
        .await
        .unwrap()
}

/// Esvazia o outbox: o que o preparo do teste emitiu.
pub async fn clear(db: &PgPool) {
    sqlx::query("delete from outbox").execute(db).await.unwrap();
}

pub fn tenant(slug: &str) -> Value {
    json!({ "kind": "tenant", "tenant": slug })
}

pub fn tournament(slug: &str, key: Option<&str>) -> Value {
    json!({ "kind": "tournament", "tenant": "quake-br", "tournament": slug, "match": key })
}

pub fn me(users: &[i64]) -> Value {
    json!({ "kind": "me", "users": users })
}

/// Os avisos do outbox como `(code, destinatários, params)`.
pub fn notices(events: &[Value]) -> Vec<(String, Vec<i64>, Value)> {
    events
        .iter()
        .filter(|e| e["kind"] == "notice")
        .map(|e| {
            let users = e["users"]
                .as_array()
                .unwrap()
                .iter()
                .map(|u| u.as_i64().unwrap())
                .collect();
            let notice = &e["notice"];
            (
                notice["code"].as_str().unwrap().to_owned(),
                users,
                notice["params"].clone(),
            )
        })
        .collect()
}

/// Os ids em ordem, como os eventos guardam os destinatários.
pub fn ids(users: &[&crate::common::User]) -> Vec<i64> {
    let mut ids: Vec<i64> = users.iter().map(|u| u.id).collect();
    ids.sort_unstable();
    ids
}
