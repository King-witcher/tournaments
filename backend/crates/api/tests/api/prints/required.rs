//! Print obrigatório: o torneio liga (só com o bucket), e o envio do jogador
//! passa a exigir um print anexado ao mapa. A organização lança sem print, e
//! sem o bucket a regra não vale.

use axum::http::StatusCode;
use serde_json::json;

use super::*;
use crate::common::*;
use crate::matches::{COPA, game as map, report, set_game};

#[tokio::test]
async fn turning_it_on_needs_the_bucket() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let on = json!({ "prints_required": true });
    let (status, body) = post(
        &app,
        "/tenants/quake-br/tournaments",
        &ana,
        with(duel("copa"), on.clone()),
    )
    .await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("tournament.prints_unavailable")
        )
    );
    let created = create_tournament(&app, &ana, duel("copa")).await;
    assert_eq!(created["prints_required"], false);
    let (status, body) = patch(&app, COPA, &ana, on.clone()).await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("tournament.prints_unavailable")
        )
    );
    // Desligado continua passando (o formulário manda tudo de novo).
    let (status, _) = patch(&app, COPA, &ana, json!({ "prints_required": false })).await;
    assert_eq!(status, StatusCode::OK);

    let (with_bucket, _, _) = app_with_bucket(&db).await;
    let (status, body) = patch(&with_bucket, COPA, &ana, on.clone()).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["prints_required"], true);
    // O que já está ligado continua aceito sem o bucket.
    let (status, _) = patch(&app, COPA, &ana, on).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn the_player_reports_only_with_a_print() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cup(&db, 3, json!({ "prints_required": true })).await;
    let list = listing(&c.app, &c.p1, FINAL).await;
    assert_eq!(list["required"], true);
    let (status, body) = report(&c.app, &c.p1, FINAL, 1, map("a", "awoken")).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("match.print_required"))
    );
    // Um upload pedido e não registrado não conta.
    let (_, _) = request_upload(&c.app, &c.p1, FINAL, 1, "image/webp", 10).await;
    let (status, _) = report(&c.app, &c.p1, FINAL, 1, map("a", "awoken")).await;
    assert_eq!(status, StatusCode::CONFLICT);
    // O print de qualquer um dos dois serve.
    let (status, body) = attach_print(&c.app, &c.p2, FINAL, 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = report(&c.app, &c.p1, FINAL, 1, map("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // O mapa seguinte precisa do dele.
    let (status, _) = report(&c.app, &c.p1, FINAL, 2, map("a", "exile")).await;
    assert_eq!(status, StatusCode::CONFLICT);
    // A organização lança sem print.
    let (status, body) = set_game(&c.app, &c.ana, FINAL, 2, map("a", "exile"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
async fn without_the_bucket_the_rule_does_not_hold() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    // Ligado quando havia bucket.
    sqlx::query("update tournaments set prints_required = true")
        .execute(&db)
        .await
        .unwrap();
    let (status, body) = report(&app, &p[0], FINAL, 1, map("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
async fn with_the_veto_the_map_comes_first_then_the_print() {
    let (db, _lease) = crate::db::test_db().await;
    let veto = json!({ "map_mode": "random", "champion_phase": false });
    let c = cup(&db, 1, json!({ "prints_required": true, "veto": veto })).await;
    // Antes do veto, nem se sabe o mapa: o print fica para depois.
    let (status, body) = report(&c.app, &c.p1, FINAL, 1, map("a", "awoken")).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("match.veto_pending"))
    );
    let ready = format!("{COPA}/matches/{FINAL}/veto/ready");
    post_empty(&c.app, &ready, &c.p1).await;
    let (status, d) = post_empty(&c.app, &ready, &c.p2).await;
    assert_eq!(status, StatusCode::OK, "{d}");
    let drawn = d["veto"]["games"][0]["map"]["slug"]
        .as_str()
        .unwrap_or_else(|| panic!("sem o mapa sorteado: {d}"))
        .to_owned();
    // Com o mapa do veto, falta o print.
    let (status, body) = report(&c.app, &c.p1, FINAL, 1, map("a", &drawn)).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("match.print_required"))
    );
    let (status, body) = attach_print(&c.app, &c.p1, FINAL, 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // Outro mapa que o do veto continua recusado, com print ou sem.
    let other = if drawn == "awoken" { "exile" } else { "awoken" };
    let (status, body) = report(&c.app, &c.p1, FINAL, 1, map("a", other)).await;
    assert_eq!(
        (status, body),
        (StatusCode::UNPROCESSABLE_ENTITY, err("match.map_mismatch"))
    );
    let (status, body) = report(&c.app, &c.p1, FINAL, 1, map("a", &drawn)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}
