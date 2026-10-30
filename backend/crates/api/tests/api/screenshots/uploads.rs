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
        "/screenshots/settings",
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
        "/screenshots/settings",
        None,
        None,
    )
    .await;
    assert_eq!(
        body,
        json!({
            "enabled": true,
            "max_bytes": 2_097_152,
            "max_per_person": 1,
            "content_types": ["image/webp", "image/jpeg"]
        })
    );
}

#[tokio::test]
async fn a_player_attaches_before_reporting_and_everyone_sees_it() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cup(&db, 3, json!({})).await;
    clear(&db).await;
    let (status, screenshot) = attach_screenshot(&c.app, &c.p1, FINAL, 1).await;
    assert_eq!(status, StatusCode::OK, "{screenshot}");
    assert_eq!(screenshot["uploaded_by"]["nickname"], "P1");
    assert_eq!(screenshot["can_remove"], true);
    // A página da partida recarrega.
    assert!(outbox(&db).await.contains(&tournament("copa", Some(FINAL))));

    // O objeto foi para o bucket na pasta da comunidade, do torneio, da
    // partida e do mapa.
    let key = object_key(&db, &screenshot["id"]).await;
    assert!(
        key.starts_with("screenshots/quake-br/copa/wb-1-1/mapa-1-"),
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

    // O adversário e quem só assiste veem o screenshot do próximo mapa, com uma
    // URL de leitura que o bucket aceita.
    for who in [&c.p2, &c.bia] {
        let list = listing(&c.app, who, FINAL).await;
        assert_eq!(list["enabled"], true);
        assert_eq!(list["required"], false);
        assert_eq!(numbers(&list), [1]);
        let shown = &game(&list, 1)["screenshots"][0];
        assert_eq!(shown["id"], screenshot["id"]);
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

    // Enviado o mapa 1, ele mantém o screenshot e o mapa 2 aparece.
    let (status, body) = report(&c.app, &c.p1, FINAL, 1, map("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let list = listing(&c.app, &c.p2, FINAL).await;
    assert_eq!(numbers(&list), [1, 2]);
    assert_eq!(game(&list, 1)["screenshots"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn without_a_bucket_there_are_no_screenshots() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    let (status, body) = request_upload(&app, &p[0], FINAL, 1, "image/webp", 1000).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body, err("screenshots.not_configured"));
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
        (StatusCode::FORBIDDEN, err("screenshots.forbidden"))
    );
    // Só o próximo mapa ainda não enviado.
    let (status, body) = request_upload(&c.app, &c.p1, FINAL, 2, "image/webp", 10).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("screenshots.game_unavailable"))
    );
    // A organização anexa mesmo sem jogar.
    let (status, body) = attach_screenshot(&c.app, &c.ana, FINAL, 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (status, _) = report(&c.app, &c.p1, FINAL, 1, map("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK);
    // O enviado (como prova) e o próximo.
    for n in [1, 2] {
        let (status, body) = attach_screenshot(&c.app, &c.p2, FINAL, n).await;
        assert_eq!(status, StatusCode::OK, "{n}: {body}");
    }
    let (status, _) = confirm(&c.app, &c.p2, FINAL, 1).await;
    assert_eq!(status, StatusCode::OK);
    // Confirmada pelo adversário, ainda dá para contestar (nenhum dos dois
    // começou outra série): o jogador ainda anexa.
    let (status, body) = attach_screenshot(&c.app, &c.p1, FINAL, 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // Confirmada pela organização, não se contesta mais: o jogador não anexa;
    // a organização sim (trocando o dela: um de cada pessoa).
    let (status, _) = report(&c.app, &c.p1, FINAL, 2, map("b", "exile")).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = confirm(&c.app, &c.ana, FINAL, 2).await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = request_upload(&c.app, &c.p1, FINAL, 2, "image/webp", 10).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("screenshots.game_confirmed"))
    );
    let list = listing(&c.app, &c.ana, FINAL).await;
    let staff = &game(&list, 1)["screenshots"][0];
    assert_eq!(staff["uploaded_by"]["nickname"], "Ana");
    let uri = format!("{}/{}", screenshots_uri(FINAL, 1), staff["id"]);
    assert_eq!(delete(&c.app, &uri, &c.ana).await.0, StatusCode::NO_CONTENT);
    let (status, body) = attach_screenshot(&c.app, &c.ana, FINAL, 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let list = listing(&c.app, &c.p1, FINAL).await;
    assert_eq!(game(&list, 2)["can_attach"], false);
    assert_eq!(game(&list, 1)["screenshots"].as_array().unwrap().len(), 3);
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
        (StatusCode::CONFLICT, err("screenshots.upload_missing"))
    );
    // Só quem pediu registra.
    let sent = put_to_bucket(&upload, "image/webp", vec![7; 1000]).await;
    assert_eq!(sent, StatusCode::OK);
    // A URL grava uma vez: o arquivo conferido não é trocado por outro do
    // mesmo tamanho enquanto ela vale.
    let again = put_to_bucket(&upload, "image/webp", vec![8; 1000]).await;
    assert_eq!(again, StatusCode::PRECONDITION_FAILED);
    let key = object_key(&db, &upload["id"]).await;
    assert_eq!(c.s3.object(&key).unwrap().body, vec![7; 1000]);
    let (status, body) = attach(&c.app, &c.p2, FINAL, 1, &upload["id"]).await;
    assert_eq!(
        (status, body),
        (StatusCode::NOT_FOUND, err("screenshots.not_found"))
    );
    let (status, body) = attach(&c.app, &c.p1, FINAL, 2, &upload["id"]).await;
    assert_eq!(
        (status, body),
        (StatusCode::NOT_FOUND, err("screenshots.not_found"))
    );
    let (status, _) = attach(&c.app, &c.p1, FINAL, 1, &upload["id"]).await;
    assert_eq!(status, StatusCode::OK);
    // Uma vez.
    let (status, _) = attach(&c.app, &c.p1, FINAL, 1, &upload["id"]).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // O que chegou diferente do pedido (por outro caminho) é descartado.
    let (_, upload) = request_upload(&c.app, &c.p2, FINAL, 1, "image/jpeg", 500).await;
    let key = object_key(&db, &upload["id"]).await;
    assert!(key.ends_with(".jpg"), "{key}");
    c.s3.put(&key, "image/jpeg", vec![1; 400]);
    let (status, body) = attach(&c.app, &c.p2, FINAL, 1, &upload["id"]).await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("screenshots.upload_invalid")
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
            json!({ "error": "screenshots.size_invalid", "params": { "max": 2_097_152 } })
        );
    }
    let (status, _) = request_upload(&c.app, &c.p1, FINAL, 1, "image/png", 10).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Um de cada pessoa por mapa: o do adversário não tira a vaga de ninguém.
    let limit = json!({ "error": "screenshots.limit_reached", "params": { "max": 1 } });
    let (_, early) = request_upload(&c.app, &c.p1, FINAL, 1, "image/webp", 10).await;
    assert_eq!(
        put_to_bucket(&early, "image/webp", vec![1; 10]).await,
        StatusCode::OK
    );
    for who in [&c.p1, &c.p2, &c.ana] {
        let (status, body) = attach_screenshot(&c.app, who, FINAL, 1).await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    let (status, body) = request_upload(&c.app, &c.p1, FINAL, 1, "image/webp", 10).await;
    assert_eq!((status, body), (StatusCode::CONFLICT, limit.clone()));
    // O pedido antes de anexar o primeiro também não registra o segundo.
    let (status, body) = attach(&c.app, &c.p1, FINAL, 1, &early["id"]).await;
    assert_eq!((status, body), (StatusCode::CONFLICT, limit));
    for who in [&c.p1, &c.ana] {
        let list = listing(&c.app, who, FINAL).await;
        assert_eq!(game(&list, 1)["can_attach"], false);
        assert_eq!(game(&list, 1)["screenshots"].as_array().unwrap().len(), 3);
    }
    // Em outro mapa, de novo.
    let (status, _) = report(&c.app, &c.p1, FINAL, 1, map("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = attach_screenshot(&c.app, &c.p1, FINAL, 2).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
async fn uploads_per_hour_are_limited_even_when_removed() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cup(&db, 3, json!({})).await;
    // Pedir, mandar e remover não devolve o pedido à conta da hora.
    for i in 0..30 {
        let (status, upload) = request_upload(&c.app, &c.p1, FINAL, 1, "image/webp", 10).await;
        assert_eq!(status, StatusCode::CREATED, "{i}: {upload}");
        if i % 2 == 0 {
            let uri = format!("{}/{}", screenshots_uri(FINAL, 1), upload["id"]);
            assert_eq!(delete(&c.app, &uri, &c.p1).await.0, StatusCode::NO_CONTENT);
        }
    }
    let (status, body) = request_upload(&c.app, &c.p1, FINAL, 1, "image/webp", 10).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        body,
        json!({ "error": "screenshots.too_many_uploads", "params": { "max": 30 } })
    );
    // O recusado não conta.
    let counted: Vec<i32> = sqlx::query_scalar("select uploads from screenshot_upload_counts")
        .fetch_all(&db)
        .await
        .unwrap();
    assert_eq!(counted, [30]);
    // A conta é de cada pessoa, e os pendentes não aparecem nem tiram a vaga
    // de ninguém no mapa.
    let list = listing(&c.app, &c.p2, FINAL).await;
    assert_eq!(game(&list, 1)["screenshots"], json!([]));
    let (status, _) = attach_screenshot(&c.app, &c.p2, FINAL, 1).await;
    assert_eq!(status, StatusCode::OK);
}
