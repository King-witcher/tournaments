//! Testes de `tournaments`; data de início e janela de check-in em
//! `schedule`, janela de contestação em `contest`, criação com traduções e
//! publicação em `create`.

mod contest;
mod create;
mod schedule;

use axum::http::StatusCode;
use serde_json::{Value, json};

use crate::common::*;

fn slugs(list: &Value) -> Vec<&str> {
    list.as_array()
        .unwrap()
        .iter()
        .map(|t| t["slug"].as_str().unwrap())
        .collect()
}

#[tokio::test]
async fn draft_is_only_for_organizers_until_published() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let created = create_tournament(&app, &ana, duel("copa")).await;
    assert_eq!(created["status"], "draft");
    assert_eq!(created["name"], "Copa de Outubro");
    assert_eq!(created["rules"], "- md3\n- md5 na grande final");
    assert_eq!(created["stages"], duel("copa")["stages"]);
    assert_eq!(created["translations"], json!([]));

    let (_, list) = get(&app, "/tenants/quake-br/tournaments", &ana).await;
    assert_eq!(slugs(&list), ["copa"]);
    let (_, list) = get(&app, "/tenants/quake-br/tournaments", &bia).await;
    assert_eq!(slugs(&list), Vec::<&str>::new());
    assert_eq!(
        get(&app, "/tenants/quake-br/tournaments/copa", &bia).await,
        (StatusCode::NOT_FOUND, err("tournament.not_found"))
    );

    let (status, published) =
        post_empty(&app, "/tenants/quake-br/tournaments/copa/publish", &ana).await;
    assert_eq!(status, StatusCode::OK, "{published}");
    assert_eq!(published["status"], "registration");
    assert_eq!(
        post_empty(&app, "/tenants/quake-br/tournaments/copa/publish", &ana).await,
        (StatusCode::CONFLICT, err("tournament.not_draft"))
    );

    let (status, seen) = get(&app, "/tenants/quake-br/tournaments/copa", &bia).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(seen, published);
    let (_, list) = get(&app, "/tenants/quake-br/tournaments", &bia).await;
    assert_eq!(slugs(&list), ["copa"]);
    assert_eq!(list[0]["status"], "registration");

    // Quem é de fora não descobre o tenant nem o torneio.
    let caio = user(&db, "Caio").await;
    assert_eq!(
        get(&app, "/tenants/quake-br/tournaments/copa", &caio).await,
        (StatusCode::NOT_FOUND, err("tenant.not_found"))
    );
}

#[tokio::test]
async fn stage_and_champion_rules_are_stored_as_engine_json() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let body = with(
        duel("copa"),
        json!({ "champion_rules": { "bans_per_series": 1, "mirror": false, "same_team": true, "no_repeat": { "who": "player", "scope": "series" } } }),
    );
    create_tournament(&app, &ana, body).await;

    let config = sqlx::query_scalar!("select config from tournament_stages where position = 1")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(
        config,
        json!({
            "format": "elimination",
            "kind": { "kind": "double", "grand_final_reset": true },
            "series": { "default": 3, "finals": 5 }
        })
    );
    let rules = sqlx::query_scalar!("select champion_rules from tournaments")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(
        rules,
        json!({
            "bans_per_series": 1,
            "mirror": false,
            "same_team": true,
            "no_repeat": { "who": "player", "scope": "series" }
        })
    );
}

#[tokio::test]
async fn champion_pool_check_blocks_with_numbers() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let no_repeat = |scope: &str| json!({ "champion_rules": { "bans_per_series": 1, "mirror": true, "same_team": false, "no_repeat": { "who": "player", "scope": scope } } });

    // Dupla de 16, md3/md5: 7 * 3 + 2 * 5 = 31 mapas no pior caso, + 1 ban de
    // cada lado.
    let (status, body) = post(
        &app,
        "/tenants/quake-br/tournaments",
        &ana,
        with(duel("copa"), no_repeat("tournament")),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        body,
        json!({ "error": "champions.pool_exhausted", "params": { "required": 33, "available": 16 } })
    );

    // Com 4 vagas, simples md3/md5: 3 + 5 + 2 bans = 10 cabe nos 16.
    let small = with(duel("copinha"), no_repeat("tournament"));
    let small = with(
        small,
        json!({
            "max_participants": 4,
            "stages": [{ "format": "elimination", "bracket": "single", "best_of": 3, "finals_best_of": 5 }]
        }),
    );
    let created = create_tournament(&app, &ana, small).await;
    assert_eq!(created["stages"][0]["grand_final_reset"], false);

    // Aumentar as vagas refaz a checagem: com 32 vagas são 3 * 4 + 5 + 2 = 19
    // (um ban de cada lado).
    let (status, body) = patch(
        &app,
        "/tenants/quake-br/tournaments/copinha",
        &ana,
        json!({ "max_participants": 32 }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        body,
        json!({ "error": "champions.pool_exhausted", "params": { "required": 19, "available": 16 } })
    );

    // Campeão que sai do jogo diminui o pool; publicar checa de novo.
    sqlx::query!("update champions set active = false where slug in ('anarki', 'athena', 'clutch', 'eisen', 'galena', 'keel', 'nyx', 'ranger', 'slash')")
        .execute(&db)
        .await
        .unwrap();
    let (status, body) =
        post_empty(&app, "/tenants/quake-br/tournaments/copinha/publish", &ana).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        body,
        json!({ "error": "champions.pool_exhausted", "params": { "required": 10, "available": 7 } })
    );
}

#[tokio::test]
async fn create_validates_the_input() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let invalid = |code: &str| (StatusCode::UNPROCESSABLE_ENTITY, err(code));
    let with_params = |code: &str, params: Value| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({ "error": code, "params": params }),
        )
    };
    let try_create = |body: Value| post(&app, "/tenants/quake-br/tournaments", &ana, body);

    assert_eq!(
        try_create(duel("Copa")).await,
        invalid("tournament.slug_invalid")
    );
    assert_eq!(
        try_create(duel("new")).await,
        invalid("tournament.slug_reserved")
    );
    assert_eq!(
        try_create(with(duel("copa"), json!({ "name": "  " }))).await,
        invalid("tournament.name_invalid")
    );
    assert_eq!(
        try_create(with(duel("copa"), json!({ "rules": "x".repeat(20001) }))).await,
        with_params("tournament.rules_invalid", json!({ "max": 20000 }))
    );
    assert_eq!(
        try_create(with(duel("copa"), json!({ "max_participants": 1 }))).await,
        with_params(
            "tournament.max_participants_invalid",
            json!({ "min": 2, "max": 1024 })
        )
    );
    assert_eq!(
        try_create(with(
            duel("copa"),
            json!({ "champion_rules": { "bans_per_series": 4, "mirror": true, "same_team": false, "no_repeat": null } })
        ))
        .await,
        with_params("tournament.bans_invalid", json!({ "max": 3 }))
    );
    assert_eq!(
        try_create(with(duel("copa"), json!({ "stages": [] }))).await,
        with_params("tournament.stage_count_invalid", json!({ "max": 1 }))
    );
    let even = json!({ "stages": [{ "format": "elimination", "bracket": "single", "best_of": 2, "finals_best_of": 5 }] });
    assert_eq!(
        try_create(with(duel("copa"), even)).await,
        with_params("format.invalid_best_of", json!({ "best_of": 2 }))
    );
    assert_eq!(
        try_create(with(duel("copa"), json!({ "mode": "ffa" })))
            .await
            .1,
        err("request.invalid_body")
    );

    // Slug único no tenant, mas pode repetir em outro tenant.
    create_tournament(&app, &ana, duel("copa")).await;
    assert_eq!(
        try_create(duel("copa")).await,
        (StatusCode::CONFLICT, err("tournament.slug_taken"))
    );
    create(&app, &ana, "quake-ar").await;
    let (status, _) = post(&app, "/tenants/quake-ar/tournaments", &ana, duel("copa")).await;
    assert_eq!(status, StatusCode::CREATED);

    // Player não cria.
    assert_eq!(
        post(&app, "/tenants/quake-br/tournaments", &bia, duel("outra")).await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
}

#[tokio::test]
async fn what_can_change_depends_on_the_status() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let uri = "/tenants/quake-br/tournaments/copa";

    let (status, updated) = patch(
        &app,
        uri,
        &ana,
        json!({ "name": " Copa de Novembro ", "max_participants": 8 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(updated["name"], "Copa de Novembro");
    assert_eq!(updated["max_participants"], 8);
    assert_eq!(updated["rules"], "- md3\n- md5 na grande final");
    assert_eq!(
        patch(&app, uri, &bia, json!({ "name": "x" })).await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );

    // Depois de começar: textos sim, configuração não (reenviar a mesma pode).
    sqlx::query!("update tournaments set status = 'in_progress'")
        .execute(&db)
        .await
        .unwrap();
    let (status, _) = patch(
        &app,
        uri,
        &ana,
        json!({ "rules": "Novas regras", "max_participants": 8, "stages": duel("copa")["stages"] }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        patch(&app, uri, &ana, json!({ "max_participants": 16 })).await,
        (StatusCode::CONFLICT, err("tournament.setup_locked"))
    );
    assert_eq!(
        delete(&app, uri, &ana).await,
        (StatusCode::CONFLICT, err("tournament.not_draft"))
    );

    // Encerrado: nada muda.
    sqlx::query!("update tournaments set status = 'finished'")
        .execute(&db)
        .await
        .unwrap();
    assert_eq!(
        patch(&app, uri, &ana, json!({ "name": "x" })).await,
        (StatusCode::CONFLICT, err("tournament.finished"))
    );
    assert_eq!(
        put(
            &app,
            &format!("{uri}/translations/es"),
            &ana,
            json!({ "name": "Copa" })
        )
        .await,
        (StatusCode::CONFLICT, err("tournament.finished"))
    );
    let (_, seen) = get(&app, uri, &ana).await;
    assert_eq!(seen["rules"], "Novas regras");
}

#[tokio::test]
async fn only_drafts_are_deleted() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let uri = "/tenants/quake-br/tournaments/copa";
    assert_eq!(
        delete(&app, uri, &bia).await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
    assert_eq!(
        delete(&app, uri, &ana).await,
        (StatusCode::NO_CONTENT, Value::Null)
    );
    assert_eq!(
        delete(&app, uri, &ana).await,
        (StatusCode::NOT_FOUND, err("tournament.not_found"))
    );
    let stages = sqlx::query_scalar!(r#"select count(*) as "count!" from tournament_stages"#)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(stages, 0);
}

#[tokio::test]
async fn translations() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let uri = "/tenants/quake-br/tournaments/copa";
    let es = format!("{uri}/translations/es");

    let (status, translation) = put(
        &app,
        &es,
        &ana,
        json!({ "name": " Copa de Octubre ", "description": "Duelo **abierto**." }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{translation}");
    assert_eq!(
        translation,
        json!({ "locale": "es", "name": "Copa de Octubre", "description": "Duelo **abierto**.", "rules": "" })
    );
    let (_, seen) = get(&app, uri, &ana).await;
    assert_eq!(seen["translations"], json!([translation]));
    let (_, list) = get(&app, "/tenants/quake-br/tournaments", &ana).await;
    assert_eq!(
        list[0]["translations"],
        json!([{ "locale": "es", "name": "Copa de Octubre" }])
    );

    // Substitui a anterior.
    let (_, translation) = put(&app, &es, &ana, json!({ "name": "Copa" })).await;
    assert_eq!(translation["description"], "");

    assert_eq!(
        put(
            &app,
            &format!("{uri}/translations/pt"),
            &ana,
            json!({ "name": "Copa" })
        )
        .await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("tournament.translation_same_locale")
        )
    );
    assert_eq!(
        put(
            &app,
            &format!("{uri}/translations/en"),
            &ana,
            json!({ "name": "Cup" })
        )
        .await,
        (StatusCode::BAD_REQUEST, err("request.invalid_path"))
    );
    assert_eq!(
        put(&app, &es, &bia, json!({ "name": "Copa" })).await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );

    assert_eq!(
        delete(&app, &es, &ana).await,
        (StatusCode::NO_CONTENT, Value::Null)
    );
    assert_eq!(
        delete(&app, &es, &ana).await,
        (
            StatusCode::NOT_FOUND,
            err("tournament.translation_not_found")
        )
    );
}

#[tokio::test]
async fn invite_only_still_shows_to_every_member() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let invite_only = with(duel("copa"), json!({ "admission": "invite_only" }));
    let created = create_tournament(&app, &ana, invite_only).await;
    assert_eq!(created["admission"], "invite_only");
    assert_eq!(created["participants"], 0);
    assert_eq!(created["registered"], false);
    let open = create_tournament(&app, &ana, duel("aberta")).await;
    assert_eq!(open["admission"], "open");
    publish(&app, &ana, "copa").await;
    publish(&app, &ana, "aberta").await;
    let uri = "/tenants/quake-br/tournaments/copa";

    // Bia é membro e não está inscrita, e vê os dois.
    let (_, list) = get(&app, "/tenants/quake-br/tournaments", &bia).await;
    assert_eq!(slugs(&list), ["aberta", "copa"]);
    assert_eq!(list[1]["admission"], "invite_only");
    assert_eq!(list[1]["registered"], false);
    let (status, seen) = get(&app, uri, &bia).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(seen["admission"], "invite_only");

    // O tipo de inscrição muda até encerrar.
    let (status, updated) = patch(&app, uri, &ana, json!({ "admission": "open" })).await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(updated["admission"], "open");
    set_status(&db, "copa", "finished").await;
    assert_eq!(
        patch(&app, uri, &ana, json!({ "admission": "invite_only" })).await,
        (StatusCode::CONFLICT, err("tournament.finished"))
    );
}

#[tokio::test]
async fn slots_never_drop_below_the_registered() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let caio = user(&db, "Caio").await;
    join(&app, &caio, &invite(&app, &ana, "quake-br").await).await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    let uri = "/tenants/quake-br/tournaments/copa";
    for who in [&ana, &bia, &caio] {
        let (status, _) = post_empty(&app, &format!("{uri}/registration"), who).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    assert_eq!(
        patch(&app, uri, &ana, json!({ "max_participants": 2 })).await,
        (
            StatusCode::CONFLICT,
            json!({ "error": "tournament.max_participants_below_registered", "params": { "registered": 3 } })
        )
    );
    let (status, updated) = patch(&app, uri, &ana, json!({ "max_participants": 3 })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["participants"], 3);
}
