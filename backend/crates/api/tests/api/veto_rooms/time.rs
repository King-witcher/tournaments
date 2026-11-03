//! O tempo: o sorteio ao vencer o prazo (pela leitura, pela ação e pelo
//! agendador, sem ninguém ler), o clique atrasado, a validade e a limpeza.

use api::realtime;
use axum::http::StatusCode;
use chrono::{Duration, Utc};
use serde_json::{Value, json};

use super::*;

fn last_event(room: &Value) -> Value {
    room["veto"]["history"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()
        .clone()
}

#[tokio::test]
async fn the_step_is_drawn_when_the_time_runs_out() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let links = links_of(&app, room(1, 1)).await;
    let first = started(&app, &links).await;
    assert_eq!(first["veto"]["map_steps"], json!([]));
    // 30 s + 2 s de tolerância.
    age(&db, 33).await;
    let after = detail(&app, &links.seats[0]).await;
    let drawn = last_event(&after);
    assert_eq!(drawn["kind"], "ban");
    assert_eq!(drawn["timeout"], true);
    assert!(drawn["actor"].is_null());
    assert_eq!(after["veto"]["map_steps"].as_array().unwrap().len(), 1);
    assert_ne!(
        after["veto"]["turn"]["sides"],
        first["veto"]["turn"]["sides"]
    );
    // O seguinte tem o tempo inteiro.
    let deadline: chrono::DateTime<Utc> =
        after["veto"]["deadline"].as_str().unwrap().parse().unwrap();
    assert!(deadline > Utc::now() + Duration::seconds(25), "{deadline}");
}

#[tokio::test]
async fn several_steps_come_due_in_sequence_with_nobody_reading() {
    let (db, _lease) = crate::db::test_db().await;
    let state = state(db.clone());
    let app = app_from(state.clone());
    let links = links_of(&app, room(1, 3)).await;
    started(&app, &links).await;
    // Sem nada vencido, o agendador não faz nada.
    let tick = realtime::tick(&state, Utc::now()).await.unwrap().unwrap();
    assert_eq!((tick.veto_room_steps, tick.expired_veto_rooms), (0, 0));

    // Cada passo tem 30 s + 2 s; 66 s depois do prazo inicial dão dois sorteados.
    let now = Utc::now() + Duration::seconds(66);
    let tick = realtime::tick(&state, now).await.unwrap().unwrap();
    assert_eq!(tick.veto_room_steps, 2);
    let after = detail(&app, &links.spectator).await;
    let history = after["veto"]["history"].as_array().unwrap();
    let drawn: Vec<&Value> = history.iter().filter(|e| e["timeout"] == true).collect();
    assert_eq!(drawn.len(), 2);
    // O histórico guarda a hora em que valeu, e o prazo seguinte conta dali.
    let at = |e: &Value| {
        e["at"]
            .as_str()
            .unwrap()
            .parse::<chrono::DateTime<Utc>>()
            .unwrap()
    };
    assert_eq!(at(drawn[1]) - at(drawn[0]), Duration::seconds(32));

    // Até o fim, sem ninguém clicar: o veto nunca trava.
    let far = Utc::now() + Duration::seconds(32 * 40);
    let tick = realtime::tick(&state, far).await.unwrap().unwrap();
    assert_eq!(tick.veto_room_steps, 12);
    let done = detail(&app, &links.spectator).await;
    assert_eq!(done["veto"]["status"], "done");
    for game in done["veto"]["games"].as_array().unwrap() {
        assert!(game["pick_a"]["slug"].is_string(), "{game}");
    }
}

#[tokio::test]
async fn the_late_click_has_the_grace_and_never_lands_on_the_next_step() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let links = links_of(&app, room(1, 1)).await;
    let first = started(&app, &links).await;
    let token = on_turn(&links, &first).to_owned();
    let seen = detail(&app, &token).await;
    let options = seen["veto"]["options"].as_array().unwrap().clone();
    let shown = seen["veto"]["step"].clone();
    // Passou do prazo, dentro da tolerância: vale.
    age(&db, 31).await;
    let choice = |map: &Value| json!({ "map": map, "step": shown });
    let body = ok_step(&app, &token, choice(&options[0])).await;
    assert_eq!(last_event(&body)["timeout"], false);
    // A tela velha, depois de a vez mudar: recusa, não cai no passo seguinte.
    assert_eq!(
        step(&app, &token, choice(&options[1])).await,
        (StatusCode::CONFLICT, err("veto.step_changed"))
    );
    // Sorteado no fim do tempo, a ação da tela velha também é recusada.
    let now_token = on_turn(&links, &body).to_owned();
    let shown = detail(&app, &now_token).await["veto"]["step"].clone();
    let option = detail(&app, &now_token).await["veto"]["options"][0].clone();
    age(&db, 33).await;
    assert_eq!(
        step(&app, &now_token, json!({ "map": option, "step": shown })).await,
        (StatusCode::CONFLICT, err("veto.step_changed"))
    );
    let after = detail(&app, &now_token).await;
    assert_eq!(last_event(&after)["timeout"], true);
}

#[tokio::test]
async fn an_empty_seat_is_covered_by_the_draw_at_the_end_of_the_window() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let links = links_of(
        &app,
        json!({
            "team_size": 2,
            "best_of": 1,
            "veto": { "map_mode": "off" },
            "champion_rules": { "bans_per_series": 0 }
        }),
    )
    .await;
    // Só um de cada lado entra.
    enter(&app, &links.seats[0], "king").await;
    enter(&app, &links.seats[2], "thallis").await;
    assert_eq!(ready(&app, &links.seats[0]).await.0, StatusCode::OK);
    let (_, body) = ready(&app, &links.seats[2]).await;
    assert_eq!(body["veto"]["turn"]["action"], "team_pick");
    let names: Vec<&Value> = body["veto"]["missing"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| &m["nickname"])
        .collect();
    assert_eq!(names, ["king", "", "thallis", ""]);
    // Quem entrou escolhe; quem não apareceu fica para o sorteio.
    let slot = &detail(&app, &links.seats[0]).await["veto"]["picks"][0].clone();
    ok_step(
        &app,
        &links.seats[0],
        json!({ "champion": slot["options"][0], "game": 1 }),
    )
    .await;
    age(&db, 63).await;
    let done = detail(&app, &links.spectator).await;
    assert_eq!(done["veto"]["status"], "done");
    let picks = done["veto"]["games"][0]["team_picks"].as_array().unwrap();
    assert_eq!(picks.len(), 4);
    assert_eq!(picks.iter().filter(|p| p["timeout"] == true).count(), 3);
    // Quem entra depois recebe o nome no que foi sorteado para o lugar dele.
    let body = enter(&app, &links.seats[1], "late").await;
    let late: Vec<&Value> = body["veto"]["games"][0]["team_picks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["nickname"] == "late")
        .collect();
    assert_eq!(late.len(), 1);
}

#[tokio::test]
async fn an_expired_room_is_not_read_and_the_routine_deletes_it() {
    let (db, _lease) = crate::db::test_db().await;
    let state = state(db.clone());
    let app = app_from(state.clone());
    let links = links_of(&app, room(1, 1)).await;
    let other = links_of(&app, room(1, 1)).await;
    started(&app, &links).await;
    // O espectador guardou a sala em memória: vencida, nem assim.
    detail(&app, &links.spectator).await;
    detail(&app, &links.spectator).await;
    sqlx::query("update veto_rooms set expires_at = now() where manage_token = $1")
        .bind(&links.manage)
        .execute(&db)
        .await
        .unwrap();
    let gone = (StatusCode::NOT_FOUND, err("veto_room.not_found"));
    assert_eq!(read(&app, &links.manage).await, gone);
    assert_eq!(read(&app, &links.seats[0]).await, gone);
    assert_eq!(ready(&app, &links.seats[0]).await, gone);
    assert_eq!(
        step(&app, &links.seats[0], json!({ "map": "awoken" })).await,
        gone
    );
    // O tempo dela também não corre mais.
    let far = Utc::now() + Duration::hours(1);
    let tick = realtime::tick(&state, far).await.unwrap().unwrap();
    assert_eq!(tick.veto_room_steps, 0);
    // Vencida no `far`, as duas saem, e a cascata leva o resto.
    assert_eq!(tick.expired_veto_rooms, 1);
    assert_eq!(detail(&app, &other.manage).await["role"], "manager");
    for table in ["veto_room_seats", "veto_room_events", "veto_room_maps"] {
        let left: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "select count(*) from {table} t
             where t.room_id not in (select id from veto_rooms)"
        )))
        .fetch_one(&db)
        .await
        .unwrap();
        assert_eq!(left, 0, "{table}");
    }
    let left: i64 = sqlx::query_scalar("select count(*) from veto_room_events")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(left, 0);
    let far = Utc::now() + Duration::hours(3);
    let tick = realtime::tick(&state, far).await.unwrap().unwrap();
    assert_eq!(tick.expired_veto_rooms, 1);
}
