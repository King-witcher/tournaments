//! Quantas idas ao banco custa a sala: o clique (`cost.rs` de `matches` mede o
//! da série) e a leitura, em especial a do espectador, que não pode virar uma
//! leitura por pessoa num chat de transmissão.

use axum::http::{Method, StatusCode};
use serde_json::{Value, json};

use super::*;
use crate::matches::cost::{Meter, check};

#[tokio::test]
async fn what_a_click_and_a_read_cost() {
    let (db, _lease) = crate::db::test_db().await;
    let (pool, meter) = Meter::pool(&db).await;
    let app = app(pool);
    let links = links_of(&app, room(1, 3)).await;
    let first = started(&app, &links).await;

    let token = on_turn(&links, &first).to_owned();
    let uri = format!("/veto-rooms/{token}");
    let mut micros = Vec::new();
    let mut last = None;
    for _ in 0..11 {
        let ((status, _), cost) = meter
            .measure(call(&app, Method::GET, &uri, None, None))
            .await;
        assert_eq!(status, StatusCode::OK);
        micros.push(cost.micros);
        last = Some(cost);
    }
    check("GET sala", &last.unwrap(), micros, 2);

    let mut micros = Vec::new();
    let mut last = None;
    let mut room = first;
    for _ in 0..6 {
        let token = on_turn(&links, &room).to_owned();
        let seen = detail(&app, &token).await;
        let uri = format!("/veto-rooms/{token}/steps");
        let body = json!({ "map": seen["veto"]["options"][0], "step": seen["veto"]["step"] });
        let ((status, after), cost) = meter
            .measure(call(&app, Method::POST, &uri, None, Some(body)))
            .await;
        assert_eq!(status, StatusCode::OK, "{after}");
        micros.push(cost.micros);
        last = Some(cost);
        room = after;
    }
    // A sala travada, o histórico, o evento, o próximo prazo e o commit.
    check("POST passo da sala", &last.unwrap(), micros, 5);
}

/// Conta pelos pings (cada conexão tirada do pool), não pelas queries: o log que
/// as conta pode ficar mudo com outro teste medindo ao mesmo tempo, e zero
/// queries passaria sem provar nada.
#[tokio::test]
async fn one_read_serves_every_spectator_until_something_changes() {
    let (db, _lease) = crate::db::test_db().await;
    let (pool, meter) = Meter::pool(&db).await;
    let app = app(pool);
    let links = links_of(&app, room(1, 1)).await;
    let uri = format!("/veto-rooms/{}", links.spectator);
    let get = || call(&app, Method::GET, &uri, None, None);

    let (first, cost) = meter.measure(get()).await;
    assert_eq!(first.0, StatusCode::OK);
    assert!(cost.pings > 0, "a primeira leitura vai ao banco");
    // Muitos espectadores: nenhuma ida ao banco, e a hora é a de agora.
    let mut seen: Vec<Value> = Vec::new();
    for _ in 0..50 {
        let ((status, body), cost) = meter.measure(get()).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(cost.pings, 0);
        seen.push(body);
    }
    assert_eq!(seen[0]["role"], "spectator");
    assert_ne!(seen[0]["now"], first.1["now"]);
    let without_now = |v: &Value| {
        let mut v = v.clone();
        v.as_object_mut().unwrap().remove("now");
        v
    };
    assert_eq!(without_now(&seen[49]), without_now(&first.1));

    // Uma mudança: a próxima lê uma vez, e as seguintes voltam à memória, já com
    // ela.
    enter(&app, &links.seats[0], "king").await;
    let (after, cost) = meter.measure(get()).await;
    assert_eq!(after.1["seats"][0]["name"], "king");
    assert!(cost.pings > 0, "depois da mudança, lê de novo");
    for _ in 0..10 {
        let ((_, body), cost) = meter.measure(get()).await;
        assert_eq!(cost.pings, 0);
        assert_eq!(body["seats"][0]["name"], "king");
    }

    // Quem joga e a gestão leem do banco, cada um com o seu; o que muda direto
    // no banco só chega ao espectador com a mudança seguinte.
    sqlx::query("update veto_rooms set expires_at = expires_at + interval '1 minute'")
        .execute(&db)
        .await
        .unwrap();
    let (kept, _) = meter.measure(get()).await;
    assert_eq!(kept.1["expires_at"], after.1["expires_at"]);
    let manage = format!("/veto-rooms/{}", links.manage);
    let (read, cost) = meter
        .measure(call(&app, Method::GET, &manage, None, None))
        .await;
    assert!(cost.pings > 0);
    assert_ne!(read.1["expires_at"], kept.1["expires_at"]);
    assert!(read.1["links"].is_object());
    assert!(kept.1["links"].is_null());
}
