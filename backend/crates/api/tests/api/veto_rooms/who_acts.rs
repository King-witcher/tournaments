//! Quem age pelo time na sala: quantos links ela tem, o capitão fazendo tudo pelo
//! lado (com os lugares sem link e sem nome), todos os lugares marcando "Pronto"
//! na sala de todos e "Começar agora", só da gestão.

use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use serde_json::{Value, json};

use super::*;
use crate::schema::{CHECK, exec, rejected, rule};

/// Sem veto de mapas e sem bans, para chegar logo aos picks.
fn picks_only(team_size: i16, best_of: i16, pick_order: &str, who_acts: &str) -> Value {
    json!({
        "team_size": team_size,
        "best_of": best_of,
        "who_acts": who_acts,
        "veto": { "map_mode": "off", "pick_order": pick_order },
        "champion_rules": { "bans_per_series": 0 }
    })
}

fn without_the_grid(body: &Value) -> String {
    let mut body = body.clone();
    let veto = body["veto"].as_object_mut().unwrap();
    veto.remove("champions");
    veto.remove("picks");
    veto.remove("options");
    body.to_string()
}

#[tokio::test]
async fn links_one_per_team_with_the_captain_and_one_per_seat_with_everyone() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    for size in [2, 3, 4] {
        let captain = links_of(&app, picks_only(size, 1, "alternating", "captain")).await;
        assert_eq!(captain.seats.len(), 2, "{size}v{size}");
        let room = detail(&app, &captain.manage).await;
        // O elenco do `engine` continua com todos os lugares.
        assert_eq!(room["seats"].as_array().unwrap().len(), 2 * size as usize);
        assert_eq!(room["who_acts"], "captain");
        assert_eq!(room["veto"]["nameless"], true);
        let everyone = links_of(&app, picks_only(size, 1, "alternating", "all")).await;
        assert_eq!(everyone.seats.len(), 2 * size as usize, "{size}v{size}");
        let room = detail(&app, &everyone.manage).await;
        assert_eq!(room["who_acts"], "all");
        assert_eq!(room["veto"]["nameless"], false);
    }
    // No duelo não muda nada, e o que veio é ignorado.
    let duel = links_of(&app, picks_only(1, 1, "alternating", "captain")).await;
    assert_eq!(duel.seats.len(), 2);
    assert_eq!(detail(&app, &duel.manage).await["who_acts"], "all");
    // Sem dizer nada, todos.
    let plain = links_of(&app, room(2, 1)).await;
    assert_eq!(plain.seats.len(), 4);
    // No banco, os lugares sem link ficam sem token, e não há como achá-los.
    let empty: i64 = sqlx::query_scalar("select count(*) from veto_room_seats where token is null")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(empty, 2 + 4 + 6);
}

#[tokio::test]
async fn the_captain_marks_ready_once_per_side_and_picks_for_every_seat() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let links = links_of(&app, picks_only(2, 1, "alternating", "captain")).await;
    let [a, b] = [&links.seats[0], &links.seats[1]];
    enter(&app, a, "king").await;
    enter(&app, b, "thallis").await;
    let (status, body) = ready(&app, a).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["veto"]["ready_a"], true);
    assert_eq!(body["veto"]["ready_by"], "captain");
    assert_eq!(body["veto"]["ready_waiting"].as_array().unwrap().len(), 1);
    let (status, body) = ready(&app, b).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["veto"]["status"], "in_progress");

    // O capitão do lado da vez escolhe por cada lugar: sem dizer qual, o próximo.
    let token = on_turn_of(&body, a, b);
    let seen = detail(&app, token).await;
    assert_eq!(seen["veto"]["can_act"]["pick_for_team"], true);
    let slots = seen["veto"]["picks"].as_array().unwrap();
    assert_eq!(slots.len(), 2);
    let (first, second) = (slots[0]["player"].clone(), slots[1]["player"].clone());
    let body = ok_step(&app, token, json!({ "champion": "anarki" })).await;
    assert_eq!(body["veto"]["games"][0]["team_picks"][0]["user_id"], first);

    // O resto, cada um na vez dele, até acabar.
    let mut left = 3;
    let mut body = body;
    let mut champions = ["athena", "clutch", "doom-slayer"].into_iter();
    while left > 0 {
        let token = on_turn_of(&body, a, b);
        body = ok_step(
            &app,
            token,
            json!({ "champion": champions.next().unwrap() }),
        )
        .await;
        left -= 1;
    }
    let veto = &body["veto"];
    assert_eq!(veto["status"], "done");
    let picks = veto["games"][0]["team_picks"].as_array().unwrap();
    assert_eq!(picks.len(), 4);
    assert!(picks.iter().any(|p| p["user_id"] == second));
    // Quem não tem link não tem nome.
    assert_eq!(picks.iter().filter(|p| p["nickname"] == "").count(), 2);
}

fn on_turn_of<'a>(body: &Value, a: &'a str, b: &'a str) -> &'a str {
    if body["veto"]["turn"]["sides"][0] == "a" {
        a
    } else {
        b
    }
}

#[tokio::test]
async fn blind_with_the_captain_hides_the_side_from_the_other_captain() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let links = links_of(&app, picks_only(2, 1, "blind", "captain")).await;
    let [a, b] = [&links.seats[0], &links.seats[1]];
    enter(&app, a, "king").await;
    enter(&app, b, "thallis").await;
    ready(&app, a).await;
    ready(&app, b).await;
    let seen = detail(&app, a).await;
    let slots = seen["veto"]["picks"].as_array().unwrap();
    assert_eq!(slots.len(), 2);
    let colleague = slots[1]["player"].as_i64().unwrap();
    ok_step(
        &app,
        a,
        json!({ "champion": "anarki", "game": 1, "player": colleague }),
    )
    .await;
    assert!(without_the_grid(&detail(&app, a).await).contains("anarki"));
    for token in [b, &links.spectator, &links.manage] {
        let seen = detail(&app, token).await;
        assert!(!without_the_grid(&seen).contains("anarki"), "{seen}");
        assert_eq!(
            seen["veto"]["games"][0]["hidden_picks"][0]["user_id"],
            colleague
        );
    }
}

#[tokio::test]
async fn with_everyone_each_seat_marks_and_picks_only_its_own() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let links = links_of(&app, picks_only(2, 1, "blind", "all")).await;
    // Um lugar vazio segura a sala: é a gestão que começa sem ele.
    enter(&app, &links.seats[0], "king").await;
    enter(&app, &links.seats[2], "thallis").await;
    enter(&app, &links.seats[3], "toxjq").await;
    assert_eq!(ready(&app, &links.seats[0]).await.0, StatusCode::OK);
    assert_eq!(ready(&app, &links.seats[2]).await.0, StatusCode::OK);
    let (status, body) = ready(&app, &links.seats[3]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["veto"]["ready_a"], false);
    assert_eq!(body["veto"]["ready_b"], true);
    assert_eq!(body["veto"]["status"], "waiting_ready");
    let waiting = &body["veto"]["ready_waiting"];
    assert_eq!(waiting.as_array().unwrap().len(), 1);
    assert_eq!(waiting[0]["nickname"], "");
    assert_eq!(waiting[0]["side"], "a");

    // Quem não entrou marca depois de entrar, e a sala começa.
    enter(&app, &links.seats[1], "late").await;
    let (status, body) = ready(&app, &links.seats[1]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["veto"]["status"], "in_progress");

    // Só o próprio campeão.
    let other = detail(&app, &links.seats[0]).await["seats"][1]["id"].clone();
    assert_eq!(
        step(
            &app,
            &links.seats[0],
            json!({ "champion": "anarki", "game": 1, "player": other })
        )
        .await,
        (StatusCode::FORBIDDEN, err("veto.not_your_pick"))
    );
    ok_step(
        &app,
        &links.seats[0],
        json!({ "champion": "anarki", "game": 1 }),
    )
    .await;
}

#[tokio::test]
async fn begin_now_is_only_for_the_manage_link() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let links = links_of(&app, picks_only(2, 1, "alternating", "captain")).await;
    let not_manager = (StatusCode::FORBIDDEN, err("veto_room.not_manager"));
    assert_eq!(
        act(&app, &links.spectator, "begin", None).await,
        not_manager
    );
    assert_eq!(act(&app, &links.seats[0], "begin", None).await, not_manager);
    assert_eq!(
        act(&app, "2222222222222222222222", "begin", None).await,
        (StatusCode::NOT_FOUND, err("veto_room.not_found"))
    );
    assert_eq!(
        detail(&app, &links.manage).await["veto"]["status"],
        "waiting_ready"
    );

    // A gestão começa, com quem falta para o sorteio, e a sala passa a valer 24 h.
    let body = begin(&app, &links.manage).await;
    assert_eq!(body["veto"]["status"], "in_progress");
    assert_eq!(body["role"], "manager");
    assert_eq!(body["veto"]["options"], json!([]));
    assert_eq!(body["veto"]["picks"], json!([]));
    let expires: DateTime<Utc> = body["expires_at"].as_str().unwrap().parse().unwrap();
    assert!(expires - Utc::now() > Duration::hours(23));
    assert_eq!(
        act(&app, &links.manage, "begin", None).await,
        (StatusCode::CONFLICT, err("veto.already_begun"))
    );
}

#[tokio::test]
async fn begin_now_works_in_a_duel_and_respects_who_starts() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let body = json!({
        "team_size": 1,
        "best_of": 1,
        "who_starts": "a",
        "veto": { "map_mode": "veto" },
        "champion_rules": { "bans_per_series": 0 }
    });
    let links = links_of(&app, body).await;
    let body = begin(&app, &links.manage).await;
    assert_eq!(body["veto"]["turn"]["sides"], json!(["a"]));
    let history = body["veto"]["history"].as_array().unwrap();
    assert_eq!(history[0]["kind"], "begin");
    assert_eq!(history[0]["side"], "a");
    assert_eq!(history[0]["actor"], Value::Null);
}

#[tokio::test]
async fn the_links_follow_who_acts_in_the_database() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    links_of(&app, room(2, 1)).await;
    // Todos sem token não, e o capitão com link em todos também não.
    let drop_token = "update veto_room_seats set token = null where position = 2";
    assert_eq!(
        rejected(exec(&db, drop_token).await),
        rule(CHECK, "veto_room_seats_frozen")
    );
    assert_eq!(
        rejected(exec(&db, "update veto_rooms set team_actors = 'captain'").await),
        rule(CHECK, "veto_rooms_frozen")
    );
    let insert = |actors: &str, second: &str| {
        format!(
            "do $$ declare r bigint; begin
                 insert into veto_rooms
                     (manage_token, spectator_token, team_size, best_of, map_mode, veto_order,
                      champion_phase, bans_per_series, mirror, same_team, step_seconds,
                      team_pick_seconds, expires_at, team_actors)
                 values ('2222222222222222222222', '3333333333333333333333', 2, 1, 'off',
                         'bans_first', true, 0, true, false, 30, 60, now() + interval '1 hour',
                         '{actors}')
                 returning id into r;
                 insert into veto_room_seats (room_id, side, position, token) values
                     (r, 'a', 1, '4444444444444444444444'),
                     (r, 'a', 2, {second}),
                     (r, 'b', 1, '5555555555555555555555'),
                     (r, 'b', 2, null);
             end $$"
        )
    };
    assert_eq!(
        rejected(exec(&db, &insert("x", "null")).await),
        rule(CHECK, "veto_rooms_team_actors")
    );
    assert_eq!(
        rejected(exec(&db, &insert("all", "null")).await),
        rule(CHECK, "veto_rooms_seat_links")
    );
    assert_eq!(
        rejected(exec(&db, &insert("captain", "'6666666666666666666666'")).await),
        rule(CHECK, "veto_rooms_seat_links")
    );
    exec(&db, &insert("captain", "null")).await.unwrap();
}
