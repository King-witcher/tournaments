//! O veto até o fim: "Pronto", o duelo md3 com draft, o 2v2 com a janela de
//! campeões e as escolhas recusadas. O `engine` já cobre as regras; aqui é o
//! caminho pela sala.

use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use serde_json::{Value, json};

use super::*;

fn slugs(list: &Value) -> Vec<&str> {
    list.as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect()
}

#[tokio::test]
async fn ready_of_each_side_starts_and_the_room_is_valid_for_a_day() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let links = links_of(&app, room(2, 1)).await;
    for (i, seat) in links.seats.iter().enumerate() {
        // Um lugar de cada lado fica vazio: não trava a sala.
        if i % 2 == 0 {
            enter(&app, seat, &format!("P{i}")).await;
        }
    }
    let (status, body) = ready(&app, &links.seats[0]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["veto"]["ready_a"], true);
    assert_eq!(body["veto"]["ready_b"], false);
    assert_eq!(body["veto"]["status"], "waiting_ready");
    // Repetir não muda nada.
    let (status, body) = ready(&app, &links.seats[0]).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["veto"]["history"].as_array().unwrap().len(), 1);

    let (status, body) = ready(&app, &links.seats[2]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["veto"]["status"], "in_progress");
    assert_eq!(body["veto"]["turn"]["action"], "ban");
    assert!(body["veto"]["deadline"].is_string());
    let expires: DateTime<Utc> = body["expires_at"].as_str().unwrap().parse().unwrap();
    assert!(expires - Utc::now() > Duration::hours(23));
    assert_eq!(
        ready(&app, &links.seats[2]).await,
        (StatusCode::CONFLICT, err("veto.already_begun"))
    );
    // Quem começa foi sorteado e fica no histórico.
    let history = detail(&app, &links.manage).await["veto"]["history"].clone();
    assert_eq!(history[2]["kind"], "begin");
    assert!(history[2]["side"].is_string());
}

#[tokio::test]
async fn a_duel_best_of_three_with_the_draft_to_the_end() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let links = links_of(&app, room(1, 3)).await;
    let first = started(&app, &links).await;
    // Bans primeiro: quatro bans, dois picks e o decisivo.
    assert_eq!(first["veto"]["turn"]["target"], "map");
    let (at_champions, map_steps) = run_steps(&app, &links, Some("nothing")).await;
    // 4 + 2 de mapa, 2 bans de campeão e 6 picks.
    assert_eq!(map_steps, 14);
    let done = at_champions;
    assert_eq!(done["veto"]["status"], "done");
    assert!(done["veto"]["turn"].is_null());
    assert!(done["veto"]["deadline"].is_null());
    assert_eq!(done["veto"]["map_steps"].as_array().unwrap().len(), 6);
    assert_eq!(done["veto"]["bans"].as_array().unwrap().len(), 2);
    let games = done["veto"]["games"].as_array().unwrap();
    assert_eq!(games.len(), 3);
    let maps: Vec<&str> = games
        .iter()
        .map(|g| g["map"]["slug"].as_str().unwrap())
        .collect();
    assert_ne!(maps[0], maps[1]);
    assert_ne!(maps[1], maps[2]);
    for game in games {
        assert!(game["pick_a"]["slug"].is_string(), "{game}");
        assert!(game["pick_b"]["slug"].is_string(), "{game}");
    }
    assert_eq!(done["veto"]["can_undo"], false);
    assert_eq!(done["veto"]["reportable"], false);
    // O histórico diz quem agiu, pelo nome do lugar.
    let actors: Vec<&Value> = done["veto"]["history"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "ban")
        .map(|e| &e["actor"])
        .collect();
    assert!(
        actors.iter().all(|a| *a == "P1" || *a == "P2"),
        "{actors:?}"
    );
    // Acabou: nada de novo, e o relógio não tem mais o que olhar.
    assert_eq!(
        step(&app, &links.seats[0], json!({ "map": maps[0] })).await,
        (StatusCode::CONFLICT, err("veto.finished"))
    );
    let due: Option<DateTime<Utc>> = sqlx::query_scalar("select due_at from veto_rooms")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(due, None);
    // Todos veem o mesmo fim.
    let spectator = detail(&app, &links.spectator).await;
    assert_eq!(spectator["veto"]["games"], done["veto"]["games"]);
    assert_eq!(spectator["veto"]["options"], json!([]));
}

#[tokio::test]
async fn a_duel_without_maps_is_only_the_draft() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let links = links_of(
        &app,
        json!({ "team_size": 1, "best_of": 1, "veto": { "map_mode": "off" } }),
    )
    .await;
    let first = started(&app, &links).await;
    assert_eq!(first["veto"]["turn"]["target"], "champion");
    let (done, steps) = run_steps(&app, &links, None).await;
    assert_eq!(steps, 4);
    assert_eq!(done["veto"]["status"], "done");
    assert!(done["veto"]["games"][0]["map"].is_null());
    assert!(done["veto"]["games"][0]["pick_a"]["slug"].is_string());
}

#[tokio::test]
async fn maps_only_ends_after_the_decider() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let links = links_of(
        &app,
        json!({ "team_size": 3, "best_of": 3, "veto": { "champion_phase": false } }),
    )
    .await;
    started(&app, &links).await;
    let (done, steps) = run_steps(&app, &links, None).await;
    assert_eq!(steps, 6);
    assert_eq!(done["veto"]["status"], "done");
    assert!(done["veto"]["games"][2]["map"]["slug"].is_string());
    assert_eq!(done["veto"]["champions"], json!([]));
}

#[tokio::test]
async fn a_two_on_two_with_the_champion_window() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let links = links_of(&app, room(2, 3)).await;
    started(&app, &links).await;
    let (at_window, steps) = run_steps(&app, &links, Some("team_pick")).await;
    // 6 de mapa e 2 bans de campeão.
    assert_eq!(steps, 8);
    assert_eq!(at_window["veto"]["turn"]["action"], "team_pick");
    assert_eq!(at_window["veto"]["team_pick_seconds"], 60);
    // Cada um dos 4 falta nas 3 partidas.
    let missing = at_window["veto"]["missing"].as_array().unwrap();
    assert_eq!(missing.len(), 4);
    assert_eq!(missing[0]["games"], json!([1, 2, 3]));
    assert_eq!(missing[0]["nickname"], "P1");
    let step_id = at_window["veto"]["step"].clone();

    // Qualquer ordem, um por partida; sem repetir no time, o colega não pega o
    // mesmo.
    let p1 = detail(&app, &links.seats[0]).await;
    let picks = p1["veto"]["picks"].as_array().unwrap();
    assert_eq!(picks.len(), 3);
    let athena = picks[0]["options"][0].clone();
    let chosen = json!({ "champion": athena, "game": 1, "step": step_id });
    let after = ok_step(&app, &links.seats[0], chosen.clone()).await;
    assert_eq!(after["veto"]["game"], 1);
    assert_eq!(after["veto"]["games"][0]["team_picks"][0]["nickname"], "P1");
    // A janela não abre vez nova: a tela velha continua valendo.
    assert_eq!(after["veto"]["step"], step_id);
    assert_eq!(
        step(&app, &links.seats[0], chosen.clone()).await,
        (StatusCode::CONFLICT, err("veto.already_picked"))
    );
    assert_eq!(
        step(&app, &links.seats[1], chosen.clone()).await,
        (StatusCode::CONFLICT, err("veto.champion_taken"))
    );
    let p2 = detail(&app, &links.seats[1]).await;
    let unavailable = &p2["veto"]["picks"][0]["unavailable"];
    assert!(
        unavailable
            .as_array()
            .unwrap()
            .iter()
            .any(|u| u["slug"] == athena && u["reason"] == "teammate"),
        "{unavailable}"
    );
    // A partida é obrigatória na janela; mapa não é escolha dela.
    assert_eq!(
        step(&app, &links.seats[1], json!({ "champion": athena })).await,
        (StatusCode::UNPROCESSABLE_ENTITY, err("veto.option_invalid"))
    );
    assert_eq!(
        step(&app, &links.seats[1], json!({ "map": "awoken", "game": 1 })).await,
        (StatusCode::UNPROCESSABLE_ENTITY, err("veto.option_invalid"))
    );
    // Todos escolhem tudo e a janela fecha.
    let mut last = Value::Null;
    for seat in &links.seats {
        loop {
            let seen = detail(&app, seat).await;
            let Some(slot) = seen["veto"]["picks"].as_array().unwrap().first().cloned() else {
                break;
            };
            let choice = json!({ "champion": slot["options"][0], "game": slot["game"] });
            last = ok_step(&app, seat, choice).await;
        }
    }
    assert_eq!(last["veto"]["status"], "done");
    assert_eq!(last["veto"]["missing"], json!([]));
    for game in last["veto"]["games"].as_array().unwrap() {
        assert_eq!(game["team_picks"].as_array().unwrap().len(), 4, "{game}");
        assert!(game["map"]["slug"].is_string());
    }
    // O lado vem junto da escolha, para o resultado sair lado A e lado B.
    let sides: Vec<&Value> = last["veto"]["games"][0]["team_picks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| &p["side"])
        .collect();
    assert_eq!(sides.iter().filter(|s| **s == "a").count(), 2);
}

#[tokio::test]
async fn any_player_of_the_side_acts_and_the_engine_refuses_the_rest() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let links = links_of(&app, room(2, 1)).await;
    let first = started(&app, &links).await;
    let side = first["veto"]["turn"]["sides"][0].as_str().unwrap();
    let (mine, teammate, other) = if side == "a" {
        (&links.seats[0], &links.seats[1], &links.seats[2])
    } else {
        (&links.seats[2], &links.seats[3], &links.seats[0])
    };
    let seen = detail(&app, mine).await;
    let options = slugs(&seen["veto"]["options"]);
    assert_eq!(options.len(), 7);
    // O outro lado não vê opções e não age.
    assert_eq!(detail(&app, other).await["veto"]["options"], json!([]));
    let map = json!({ "map": options[0] });
    assert_eq!(
        step(&app, other, map.clone()).await,
        (StatusCode::CONFLICT, err("veto.not_your_turn"))
    );
    // Mapa fora do pool, campeão no passo de mapa, nada, os dois.
    let invalid = (StatusCode::UNPROCESSABLE_ENTITY, err("veto.option_invalid"));
    for body in [
        json!({ "map": "nope" }),
        json!({ "champion": "athena" }),
        json!({}),
        json!({ "map": options[0], "champion": "athena" }),
    ] {
        assert_eq!(step(&app, mine, body.clone()).await, invalid, "{body}");
    }
    // O colega faz o passo do lado, e fica o nome de quem fez.
    let after = ok_step(&app, teammate, map.clone()).await;
    let history = after["veto"]["history"].as_array().unwrap();
    let last = history.last().unwrap();
    assert_eq!(last["kind"], "ban");
    assert_eq!(last["actor"], if side == "a" { "P2" } else { "P4" });
    // O mapa saiu das opções do outro lado, e banir de novo não vale.
    assert_eq!(
        step(&app, other, map).await,
        (StatusCode::UNPROCESSABLE_ENTITY, err("veto.option_invalid"))
    );
}

#[tokio::test]
async fn two_clicks_and_the_clock_at_the_same_time_do_not_race() {
    let (db, _lease) = crate::db::test_db().await;
    let state = crate::common::state(db.clone());
    let app = app_from(state.clone());
    let links = links_of(&app, room(1, 1)).await;
    for (seat, name) in links.seats.iter().zip(["king", "thallis"]) {
        enter(&app, seat, name).await;
    }
    // Os dois lados ao mesmo tempo: começa uma vez, sem um "Pronto" perdido.
    let (a, b) = tokio::join!(ready(&app, &links.seats[0]), ready(&app, &links.seats[1]));
    assert_eq!((a.0, b.0), (StatusCode::OK, StatusCode::OK), "{a:?} {b:?}");
    let first = detail(&app, &links.manage).await;
    assert_eq!(first["veto"]["status"], "in_progress");
    let kinds = |room: &Value| {
        room["veto"]["history"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["kind"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    };
    assert_eq!(kinds(&first), ["ready", "ready", "begin"]);

    // Dois cliques do mesmo lado na mesma tela: um entra, o outro é recusado.
    let token = on_turn(&links, &first);
    let seen = detail(&app, token).await;
    let options = slugs(&seen["veto"]["options"]);
    let choice = |map: &str| json!({ "map": map, "step": seen["veto"]["step"] });
    let (a, b) = tokio::join!(
        step(&app, token, choice(options[0])),
        step(&app, token, choice(options[1]))
    );
    let mut statuses = [a.0, b.0];
    statuses.sort();
    assert_eq!(
        statuses,
        [StatusCode::OK, StatusCode::CONFLICT],
        "{a:?} {b:?}"
    );
    let refused = if a.0 == StatusCode::OK { &b } else { &a };
    assert_eq!(refused.1, err("veto.step_changed"));

    // O prazo venceu: o clique, a leitura e o agendador juntos sorteiam uma vez.
    age(&db, 33).await;
    let now = chrono::Utc::now();
    let before = detail(&app, &links.manage).await;
    let token = on_turn(&links, &before);
    let (_, _, tick) = tokio::join!(
        step(&app, token, json!({ "map": "awoken", "step": 1 })),
        read(&app, token),
        api::realtime::tick(&state, now)
    );
    tick.unwrap();
    let after = detail(&app, &links.manage).await;
    let drawn = after["veto"]["history"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["timeout"] == true)
        .count();
    assert_eq!(drawn, 1);
}

#[tokio::test]
async fn the_rule_of_not_repeating_stays_inside_the_series() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let links = links_of(
        &app,
        json!({
            "team_size": 1,
            "best_of": 3,
            "veto": { "map_mode": "off" },
            "champion_rules": { "bans_per_series": 0, "no_repeat": "player" }
        }),
    )
    .await;
    started(&app, &links).await;
    let (done, _) = run_steps(&app, &links, None).await;
    let games = done["veto"]["games"].as_array().unwrap();
    let of = |side: &str| {
        let mut picks: Vec<&str> = games
            .iter()
            .map(|g| g[format!("pick_{side}")]["slug"].as_str().unwrap())
            .collect();
        picks.sort_unstable();
        picks.dedup();
        picks.len()
    };
    assert_eq!((of("a"), of("b")), (3, 3));
}
