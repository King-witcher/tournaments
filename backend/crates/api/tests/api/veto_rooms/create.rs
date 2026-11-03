//! Criar uma sala: com e sem login, os limites por hora, o teto de salas sem
//! login e a configuração recusada.

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use chrono::{DateTime, Duration, Utc};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

use super::*;

/// Cria a sala vindo do IP `ip`.
async fn create_from(app: &Router, ip: &str, who: Option<&User>) -> (StatusCode, Value) {
    let mut request = Request::post("/veto-rooms")
        .header(header::CONTENT_TYPE, "application/json")
        .header("cf-connecting-ip", ip);
    if let Some(who) = who {
        request = request.header(header::COOKIE, &who.cookie);
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(room(1, 1).to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&body).unwrap())
}

#[tokio::test]
async fn anyone_creates_a_room_with_one_link_per_player() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    for (team_size, seats) in [(1, 2), (2, 4), (3, 6), (4, 8)] {
        let links = links_of(&app, room(team_size, 3)).await;
        assert_eq!(links.seats.len(), seats);
        let d = detail(&app, &links.manage).await;
        assert_eq!(d["role"], "manager");
        assert_eq!(d["team_size"], team_size);
        assert_eq!(d["best_of"], 3);
        assert_eq!(d["seats"].as_array().unwrap().len(), seats);
        // O padrão: veto de mapas, bans primeiro, draft com um ban.
        assert_eq!(d["veto"]["map_mode"], "veto");
        assert_eq!(d["order"], "bans_first");
        assert_eq!(d["veto"]["champion_phase"], true);
        assert_eq!(d["bans_per_series"], 1);
        assert_eq!(d["veto"]["step_seconds"], 30);
        assert_eq!(d["veto"]["pool"].as_array().unwrap().len(), 7);
        assert_eq!(d["veto"]["status"], "waiting_ready");
        // Só o de gestão vê os links; cada link é o papel dele.
        let spectator = detail(&app, &links.spectator).await;
        assert_eq!(spectator["role"], "spectator");
        assert!(spectator["links"].is_null());
        let player = detail(&app, &links.seats[0]).await;
        assert_eq!(player["role"], "player");
        assert!(player["links"].is_null());
        assert_eq!(player["seat"]["side"], "a");
        assert!(player["seat"]["name"].is_null());
        // Vale 2 h se ninguém começar.
        let expires: DateTime<Utc> = d["expires_at"].as_str().unwrap().parse().unwrap();
        let left = expires - Utc::now();
        assert!(
            left > Duration::minutes(119) && left <= Duration::hours(2),
            "{left}"
        );
    }
    let anonymous: i64 =
        sqlx::query_scalar("select count(*) from veto_rooms where created_by is null")
            .fetch_one(&db)
            .await
            .unwrap();
    assert_eq!(anonymous, 4);
    let ana = user(&db, "Ana").await;
    let (status, _) = create_room(&app, Some(&ana), room(1, 1)).await;
    assert_eq!(status, StatusCode::CREATED);
    let mine: i64 = sqlx::query_scalar("select count(*) from veto_rooms where created_by = $1")
        .bind(ana.id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(mine, 1);
}

#[tokio::test]
async fn advanced_settings_are_kept() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let body = json!({
        "team_size": 2,
        "best_of": 1,
        "veto": {
            "order": "alternating",
            "map_pool": ["awoken", "insomnia", "blood-run"],
            "step_seconds": 45,
            "team_pick_seconds": 90
        },
        "champion_rules": {
            "bans_per_series": 2, "mirror": false, "same_team": true, "no_repeat": "team"
        }
    });
    let links = links_of(&app, body).await;
    let d = detail(&app, &links.manage).await;
    assert_eq!(d["order"], "alternating");
    assert_eq!(d["bans_per_series"], 2);
    assert_eq!(d["no_repeat"], "team");
    assert_eq!(d["veto"]["step_seconds"], 45);
    assert_eq!(d["veto"]["team_pick_seconds"], 90);
    let pool: Vec<&str> = d["veto"]["pool"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["slug"].as_str().unwrap())
        .collect();
    assert_eq!(pool, ["awoken", "insomnia", "blood-run"]);

    // Sem veto de mapas, o pool não importa.
    let links = links_of(
        &app,
        json!({ "team_size": 1, "best_of": 3, "veto": { "map_mode": "off" } }),
    )
    .await;
    let d = detail(&app, &links.manage).await;
    assert_eq!(d["veto"]["map_mode"], "off");
    assert_eq!(d["veto"]["pool"], json!([]));
}

#[tokio::test]
async fn settings_out_of_range_are_refused() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let with_params = |code: &str, params: Value| json!({ "error": code, "params": params });
    let veto = |veto: Value| json!({ "team_size": 1, "best_of": 3, "veto": veto });
    let rules = |rules: Value| json!({ "team_size": 1, "best_of": 3, "champion_rules": rules });
    let cases = [
        (
            room(5, 3),
            with_params(
                "tournament.team_size_invalid",
                json!({ "min": 1, "max": 4 }),
            ),
        ),
        (
            room(1, 2),
            with_params("format.invalid_best_of", json!({ "best_of": 2 })),
        ),
        (
            room(1, 7),
            with_params("format.invalid_best_of", json!({ "best_of": 7 })),
        ),
        (
            veto(json!({ "map_mode": "vote" })),
            err("veto_room.map_mode_invalid"),
        ),
        (
            veto(json!({ "map_mode": "random" })),
            err("veto_room.map_mode_invalid"),
        ),
        (
            veto(json!({ "map_mode": "off", "champion_phase": false })),
            err("veto_room.no_phases"),
        ),
        (
            veto(json!({ "map_pool": ["awoken", "insomnia"] })),
            with_params("tournament.map_pool_too_small", json!({ "min": 3 })),
        ),
        (
            veto(json!({ "map_pool": ["awoken", "insomnia", "nope"] })),
            err("tournament.map_unknown"),
        ),
        (
            veto(json!({ "step_seconds": 5 })),
            with_params(
                "tournament.veto_seconds_invalid",
                json!({ "min": 10, "max": 300 }),
            ),
        ),
        (
            rules(json!({ "bans_per_series": 4 })),
            with_params("tournament.bans_invalid", json!({ "max": 3 })),
        ),
    ];
    for (body, expected) in cases {
        let (status, got) = create_room(&app, None, body.clone()).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}: {got}");
        assert_eq!(got, expected, "{body}");
    }
    // O pior caso do pool de campeões: 3 bans de cada lado, sem repetir na série
    // de 5, sem espelho, 4 por lado, sem repetir no time. Regra do time: 6 + 16 + 3 + 4 + 1.
    let (status, got) = create_room(
        &app,
        None,
        json!({
            "team_size": 4,
            "best_of": 5,
            "champion_rules": {
                "bans_per_series": 3, "mirror": false, "same_team": false, "no_repeat": "team"
            }
        }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{got}");
    assert_eq!(got["error"], "champions.pool_exhausted");
    assert_eq!(got["params"]["required"], 30);
    // Pronto e o resto só valem com a sala; corpo que não é a sala não cria nada.
    let (status, got) = create_room(&app, None, json!({ "team_size": "dois" })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(got, err("request.invalid_body"));
}

#[tokio::test]
async fn per_hour_limits_count_the_ip_without_login_and_the_account_with_it() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    for _ in 0..30 {
        assert_eq!(
            create_from(&app, "203.0.113.7", None).await.0,
            StatusCode::CREATED
        );
    }
    let (status, body) = create_from(&app, "203.0.113.7", None).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["error"], "veto_room.rate_limited");
    assert!(body["params"]["minutes"].as_i64().unwrap() <= 60);
    // Outro IP continua; quem entra com a conta, do mesmo IP, também.
    assert_eq!(
        create_from(&app, "203.0.113.8", None).await.0,
        StatusCode::CREATED
    );
    let ana = user(&db, "Ana").await;
    for _ in 0..60 {
        assert_eq!(
            create_from(&app, "203.0.113.7", Some(&ana)).await.0,
            StatusCode::CREATED
        );
    }
    let (status, body) = create_from(&app, "203.0.113.7", Some(&ana)).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["error"], "veto_room.rate_limited");
    let bia = user(&db, "Bia").await;
    assert_eq!(
        create_from(&app, "203.0.113.7", Some(&bia)).await.0,
        StatusCode::CREATED
    );
}

/// `count` salas sem login, direto no banco.
async fn fill_anonymous(db: &PgPool, count: i32) {
    sqlx::query(
        "with rooms as (
             insert into veto_rooms
                 (manage_token, spectator_token, team_size, best_of, map_mode, veto_order,
                  champion_phase, bans_per_series, mirror, same_team, step_seconds,
                  team_pick_seconds, expires_at)
             select lpad(replace(n::text, '0', 'A'), 22, 'm'),
                    lpad(replace(n::text, '0', 'A'), 22, 's'),
                    1, 1, 'off', 'bans_first', true, 0, true, false, 30, 60,
                    now() + interval '1 hour'
             from generate_series(1, $1) n
             returning id
         )
         insert into veto_room_seats (room_id, side, position, token)
         select id, side, 1, lpad(replace(id::text, '0', 'A'), 21, 't') || upper(side)
         from rooms, (values ('a'), ('b')) as sides (side)",
    )
    .bind(count)
    .execute(db)
    .await
    .unwrap();
}

#[tokio::test]
async fn the_ceiling_stops_only_rooms_without_login() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    fill_anonymous(&db, 4_999).await;
    // A última vaga.
    assert_eq!(
        create_room(&app, None, room(1, 1)).await.0,
        StatusCode::CREATED
    );
    assert_eq!(
        create_room(&app, None, room(1, 1)).await,
        (
            StatusCode::SERVICE_UNAVAILABLE,
            err("veto_room.anonymous_full")
        )
    );
    // Conta logada nunca é barrada, e a sala dela não conta.
    let ana = user(&db, "Ana").await;
    assert_eq!(
        create_room(&app, Some(&ana), room(1, 1)).await.0,
        StatusCode::CREATED
    );
    // Sala vencida não conta: abre vaga, mesmo antes da limpeza.
    sqlx::query(
        "update veto_rooms set expires_at = now() - interval '1 second'
         where id = (select min(id) from veto_rooms where created_by is null)",
    )
    .execute(&db)
    .await
    .unwrap();
    assert_eq!(
        create_room(&app, None, room(1, 1)).await.0,
        StatusCode::CREATED
    );
}
