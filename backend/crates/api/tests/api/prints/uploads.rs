//! Pedir a URL, mandar ao bucket e registrar: o caminho do navegador, quem
//! anexa a qual mapa, o que o bucket e a API conferem e os limites.

use axum::http::StatusCode;
use serde_json::json;

use super::*;
use crate::common::*;
use crate::matches::{confirm, game as map, report};
use crate::realtime::{clear, outbox, tournament};

#[tokio::test]
async fn settings_say_whether_there_is_a_bucket() {
    let (db, _lease) = crate::db::test_db().await;
    let without = app(db.clone());
    let (status, body) = call(
        &without,
        axum::http::Method::GET,
        "/prints/settings",
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["enabled"], false);

    let (with_bucket, _, _) = app_with_bucket(&db).await;
    let (_, body) = call(
        &with_bucket,
        axum::http::Method::GET,
        "/prints/settings",
        None,
        None,
    )
    .await;
    assert_eq!(
        body,
        json!({
            "enabled": true,
            "max_bytes": 2_097_152,
            "max_per_game": 3,
            "content_types": ["image/webp", "image/jpeg"]
        })
    );
}

#[tokio::test]
async fn a_player_attaches_before_reporting_and_everyone_sees_it() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cup(&db, 3, json!({})).await;
    clear(&db).await;
    let (status, print) = attach_print(&c.app, &c.p1, FINAL, 1).await;
    assert_eq!(status, StatusCode::OK, "{print}");
    assert_eq!(print["uploaded_by"]["nickname"], "P1");
    assert_eq!(print["can_remove"], true);
    // A página da partida recarrega.
    assert!(outbox(&db).await.contains(&tournament("copa", Some(FINAL))));

    // O objeto foi para o bucket na pasta da comunidade, do torneio, da
    // partida e do mapa.
    let key = object_key(&db, &print["id"]).await;
    assert!(
        key.starts_with("prints/quake-br/copa/wb-1-1/mapa-1-"),
        "{key}"
    );
    assert!(key.ends_with(".webp"));
    assert_eq!(
        c.s3.keys().into_iter().collect::<Vec<_>>(),
        std::slice::from_ref(&key)
    );
    let sent = c.s3.object(&key).unwrap();
    assert_eq!(sent.content_type.as_deref(), Some("image/webp"));
    assert_eq!(sent.body.len(), 1000);

    // O adversário e quem só assiste veem o print do próximo mapa, com uma
    // URL de leitura que o bucket aceita.
    for who in [&c.p2, &c.bia] {
        let list = listing(&c.app, who, FINAL).await;
        assert_eq!(list["enabled"], true);
        assert_eq!(list["required"], false);
        assert_eq!(numbers(&list), [1]);
        let shown = &game(&list, 1)["prints"][0];
        assert_eq!(shown["id"], print["id"]);
        assert_eq!(shown["can_remove"], false);
        assert_eq!(shown["content_type"], "image/webp");
        let image = reqwest::get(shown["url"].as_str().unwrap()).await.unwrap();
        assert_eq!(image.status(), StatusCode::OK);
        assert_eq!(image.bytes().await.unwrap().len(), 1000);
    }
    assert_eq!(
        game(&listing(&c.app, &c.p2, FINAL).await, 1)["can_attach"],
        true
    );
    assert_eq!(
        game(&listing(&c.app, &c.bia, FINAL).await, 1)["can_attach"],
        false
    );

    // Enviado o mapa 1, ele mantém o print e o mapa 2 aparece.
    let (status, body) = report(&c.app, &c.p1, FINAL, 1, map("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let list = listing(&c.app, &c.p2, FINAL).await;
    assert_eq!(numbers(&list), [1, 2]);
    assert_eq!(game(&list, 1)["prints"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn without_a_bucket_there_are_no_prints() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    let (status, body) = request_upload(&app, &p[0], FINAL, 1, "image/webp", 1000).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body, err("prints.not_configured"));
    let list = listing(&app, &p[0], FINAL).await;
    assert_eq!(
        (&list["enabled"], &list["games"]),
        (&json!(false), &json!([]))
    );
}

#[tokio::test]
async fn players_and_staff_attach_to_sent_maps_and_the_next() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cup(&db, 3, json!({})).await;
    // Quem não joga nem organiza, não.
    let (status, body) = request_upload(&c.app, &c.bia, FINAL, 1, "image/webp", 10).await;
    assert_eq!(
        (status, body),
        (StatusCode::FORBIDDEN, err("prints.forbidden"))
    );
    // Só o próximo mapa ainda não enviado.
    let (status, body) = request_upload(&c.app, &c.p1, FINAL, 2, "image/webp", 10).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("prints.game_unavailable"))
    );
    // A organização anexa mesmo sem jogar.
    let (status, body) = attach_print(&c.app, &c.ana, FINAL, 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (status, _) = report(&c.app, &c.p1, FINAL, 1, map("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK);
    // O enviado (como prova) e o próximo.
    for n in [1, 2] {
        let (status, body) = attach_print(&c.app, &c.p2, FINAL, n).await;
        assert_eq!(status, StatusCode::OK, "{n}: {body}");
    }
    let (status, _) = confirm(&c.app, &c.p2, FINAL, 1).await;
    assert_eq!(status, StatusCode::OK);
    // Confirmado, o jogador não anexa mais; a organização sim.
    let (status, body) = request_upload(&c.app, &c.p1, FINAL, 1, "image/webp", 10).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("prints.game_confirmed"))
    );
    let (status, body) = attach_print(&c.app, &c.ana, FINAL, 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let list = listing(&c.app, &c.p1, FINAL).await;
    assert_eq!(game(&list, 1)["can_attach"], false);
    assert_eq!(game(&list, 1)["prints"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn the_bucket_and_the_api_check_the_upload() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cup(&db, 3, json!({})).await;
    let (status, upload) = request_upload(&c.app, &c.p1, FINAL, 1, "image/webp", 1000).await;
    assert_eq!(status, StatusCode::CREATED, "{upload}");
    assert_eq!(upload["content_type"], "image/webp");
    // O bucket recusa outro tamanho e outro tipo: estão na assinatura.
    let sent = put_to_bucket(&upload, "image/webp", vec![7; 1001]).await;
    assert_eq!(sent, StatusCode::FORBIDDEN);
    let sent = put_to_bucket(&upload, "image/png", vec![7; 1000]).await;
    assert_eq!(sent, StatusCode::FORBIDDEN);
    // Sem o arquivo no bucket, não registra.
    let (status, body) = attach(&c.app, &c.p1, FINAL, 1, &upload["id"]).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("prints.upload_missing"))
    );
    // Só quem pediu registra.
    let sent = put_to_bucket(&upload, "image/webp", vec![7; 1000]).await;
    assert_eq!(sent, StatusCode::OK);
    let (status, body) = attach(&c.app, &c.p2, FINAL, 1, &upload["id"]).await;
    assert_eq!(
        (status, body),
        (StatusCode::NOT_FOUND, err("prints.not_found"))
    );
    let (status, body) = attach(&c.app, &c.p1, FINAL, 2, &upload["id"]).await;
    assert_eq!(
        (status, body),
        (StatusCode::NOT_FOUND, err("prints.not_found"))
    );
    let (status, _) = attach(&c.app, &c.p1, FINAL, 1, &upload["id"]).await;
    assert_eq!(status, StatusCode::OK);
    // Uma vez.
    let (status, _) = attach(&c.app, &c.p1, FINAL, 1, &upload["id"]).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // O que chegou diferente do pedido (por outro caminho) é descartado.
    let (_, upload) = request_upload(&c.app, &c.p1, FINAL, 1, "image/jpeg", 500).await;
    let key = object_key(&db, &upload["id"]).await;
    assert!(key.ends_with(".jpg"), "{key}");
    c.s3.put(&key, "image/jpeg", vec![1; 400]);
    let (status, body) = attach(&c.app, &c.p1, FINAL, 1, &upload["id"]).await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("prints.upload_invalid")
        )
    );
    assert!(!stored(&db).await.contains(&key));
    assert_eq!(queued(&db).await, [key]);
}

#[tokio::test]
async fn sizes_and_limits() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cup(&db, 3, json!({})).await;
    for size in [0, 2_097_153] {
        let (status, body) = request_upload(&c.app, &c.p1, FINAL, 1, "image/webp", size).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            json!({ "error": "prints.size_invalid", "params": { "max": 2_097_152 } })
        );
    }
    let (status, _) = request_upload(&c.app, &c.p1, FINAL, 1, "image/png", 10).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Três por mapa.
    for who in [&c.p1, &c.p2, &c.ana] {
        let (status, body) = attach_print(&c.app, who, FINAL, 1).await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    let (status, body) = request_upload(&c.app, &c.p1, FINAL, 1, "image/webp", 10).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        body,
        json!({ "error": "prints.limit_reached", "params": { "max": 3 } })
    );
    assert_eq!(
        game(&listing(&c.app, &c.p1, FINAL).await, 1)["can_attach"],
        false
    );
}

#[tokio::test]
async fn abandoned_uploads_are_limited() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cup(&db, 3, json!({})).await;
    for _ in 0..10 {
        let (status, _) = request_upload(&c.app, &c.p1, FINAL, 1, "image/webp", 10).await;
        assert_eq!(status, StatusCode::CREATED);
    }
    let (status, body) = request_upload(&c.app, &c.p1, FINAL, 1, "image/webp", 10).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        body,
        json!({ "error": "prints.too_many_uploads", "params": { "max": 10 } })
    );
    // Os pendentes não aparecem nem contam para os três do mapa.
    let list = listing(&c.app, &c.p2, FINAL).await;
    assert_eq!(game(&list, 1)["prints"], json!([]));
    let (status, _) = attach_print(&c.app, &c.p2, FINAL, 1).await;
    assert_eq!(status, StatusCode::OK);
}
