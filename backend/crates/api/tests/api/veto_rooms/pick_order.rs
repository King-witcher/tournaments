//! A ordem das escolhas e "Quem começa" na sala de veto, e o que fica escondido às
//! cegas: um lugar do outro lado, o colega, o espectador (também pela visão em
//! memória) e a gestão, antes e depois de a janela fechar.

use axum::http::StatusCode;
use serde_json::{Value, json};

use super::*;

/// Sem veto de mapas e sem bans, para chegar logo aos picks.
fn picks_only(team_size: i16, best_of: i16, pick_order: &str) -> Value {
    json!({
        "team_size": team_size,
        "best_of": best_of,
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
async fn a_two_on_two_alternates_player_by_player() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let links = links_of(&app, picks_only(2, 1, "alternating")).await;
    let first = started(&app, &links).await;
    let veto = &first["veto"];
    assert_eq!(veto["pick_order"], "alternating");
    assert_eq!(veto["team_pick_seconds"], Value::Null);
    assert_eq!(veto["turn"]["action"], "team_pick");
    assert_eq!(veto["turn"]["pick_for"], 1);
    // Do lado sorteado, os dois podem; do outro, nenhum.
    let opener = veto["turn"]["sides"][0].as_str().unwrap().to_owned();
    let (open, other) = if opener == "a" {
        (&links.seats[0..2], &links.seats[2..4])
    } else {
        (&links.seats[2..4], &links.seats[0..2])
    };
    assert_eq!(detail(&app, &open[1]).await["veto"]["picks"][0]["game"], 1);
    assert_eq!(detail(&app, &other[0]).await["veto"]["picks"], json!([]));
    assert_eq!(
        step(&app, &other[0], json!({ "champion": "anarki" })).await,
        (StatusCode::CONFLICT, err("veto.not_your_turn"))
    );
    // O segundo do lado escolhe; a vez passa ao outro lado, duas vezes.
    let after = ok_step(&app, &open[1], json!({ "champion": "anarki" })).await;
    assert_ne!(after["veto"]["turn"]["sides"][0], opener);
    assert_eq!(
        step(&app, &open[0], json!({ "champion": "athena" })).await,
        (StatusCode::CONFLICT, err("veto.not_your_turn"))
    );
    ok_step(&app, &other[0], json!({ "champion": "anarki" })).await;
    let after = ok_step(&app, &other[1], json!({ "champion": "athena" })).await;
    assert_eq!(after["veto"]["turn"]["sides"][0], opener);
    // Quem já escolheu não é mais com ele.
    assert_eq!(detail(&app, &open[1]).await["veto"]["picks"], json!([]));
    assert_eq!(
        step(&app, &open[1], json!({ "champion": "clutch" })).await,
        (StatusCode::CONFLICT, err("veto.already_picked"))
    );
    let last = ok_step(&app, &open[0], json!({ "champion": "clutch" })).await;
    assert_eq!(last["veto"]["status"], "done");
    assert_eq!(
        last["veto"]["games"][0]["team_picks"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
}

#[tokio::test]
async fn each_alternating_pick_has_its_own_deadline_and_draw() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let links = links_of(&app, picks_only(2, 1, "alternating")).await;
    started(&app, &links).await;
    age(&db, 33).await;
    let d = detail(&app, &links.spectator).await;
    let picks = d["veto"]["games"][0]["team_picks"].as_array().unwrap();
    assert_eq!(picks.len(), 1, "{d}");
    assert_eq!(picks[0]["timeout"], true);
    // O primeiro do lado, na ordem dos lugares.
    assert_eq!(
        picks[0]["nickname"],
        if picks[0]["side"] == "a" { "P1" } else { "P3" }
    );
    assert!(d["veto"]["deadline"].is_string());
}

#[tokio::test]
async fn blind_hides_a_side_from_the_other_side_the_spectator_and_the_manager() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let links = links_of(&app, picks_only(2, 1, "blind")).await;
    let first = started(&app, &links).await;
    assert_eq!(first["veto"]["pick_order"], "blind");
    assert_eq!(first["veto"]["team_pick_seconds"], 60);
    assert_eq!(first["veto"]["turn"]["sides"], json!(["a", "b"]));
    assert_eq!(first["veto"]["turn"]["pick_for"], Value::Null);
    // O espectador lê antes, para a visão ficar guardada em memória.
    let kept = detail(&app, &links.spectator).await;
    assert_eq!(kept["role"], "spectator");

    let chosen = json!({ "champion": "anarki", "game": 1 });
    let mine = ok_step(&app, &links.seats[0], chosen).await;
    assert!(without_the_grid(&mine).contains("anarki"));
    // O colega vê; o motivo diz o colega.
    let mate = detail(&app, &links.seats[1]).await;
    assert!(without_the_grid(&mate).contains("anarki"));
    assert_eq!(
        mate["veto"]["picks"][0]["unavailable"][0]["reason"],
        "teammate"
    );
    // Quem não é do lado, não.
    let outsiders = [
        (&links.seats[2], "o adversário"),
        (&links.seats[3], "o outro adversário"),
        (&links.spectator, "o espectador"),
        (&links.manage, "a gestão"),
    ];
    for (token, label) in outsiders {
        let seen = detail(&app, token).await;
        let text = without_the_grid(&seen);
        assert!(!text.contains("anarki"), "{label} viu: {text}");
        let game = &seen["veto"]["games"][0];
        assert_eq!(game["team_picks"], json!([]), "{label}");
        assert_eq!(game["hidden_picks"][0]["side"], "a", "{label}");
        assert_eq!(game["hidden_picks"][0]["nickname"], "P1", "{label}");
        let history = seen["veto"]["history"].as_array().unwrap();
        assert!(history.iter().all(|e| e["kind"] != "pick"), "{label}");
    }
    // O espectador continua sem ver: a visão guardada refez sem o pick.
    assert!(!without_the_grid(&detail(&app, &links.spectator).await).contains("anarki"));
    // Com espelho, o adversário pega o mesmo sem saber.
    let slot = detail(&app, &links.seats[2]).await["veto"]["picks"][0].clone();
    assert!(
        slot["options"]
            .as_array()
            .unwrap()
            .contains(&json!("anarki"))
    );
    assert_eq!(slot["unavailable"], json!([]));
    ok_step(
        &app,
        &links.seats[2],
        json!({ "champion": "anarki", "game": 1 }),
    )
    .await;
    ok_step(
        &app,
        &links.seats[1],
        json!({ "champion": "athena", "game": 1 }),
    )
    .await;
    // O último pick fecha: a visão do espectador abre, sem esperar nada.
    let last = ok_step(
        &app,
        &links.seats[3],
        json!({ "champion": "clutch", "game": 1 }),
    )
    .await;
    assert_eq!(last["veto"]["status"], "done");
    for (token, label) in outsiders {
        let seen = detail(&app, token).await;
        let game = &seen["veto"]["games"][0];
        assert_eq!(game["team_picks"].as_array().unwrap().len(), 4, "{label}");
        assert_eq!(game["hidden_picks"], json!([]), "{label}");
        let history = seen["veto"]["history"].as_array().unwrap();
        assert_eq!(history.iter().filter(|e| e["kind"] == "pick").count(), 4);
    }
}

#[tokio::test]
async fn a_seat_nobody_joined_still_sees_as_its_side() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let links = links_of(&app, picks_only(2, 1, "blind")).await;
    enter(&app, &links.seats[0], "king").await;
    enter(&app, &links.seats[2], "thallis").await;
    assert_eq!(ready(&app, &links.seats[0]).await.0, StatusCode::OK);
    assert_eq!(ready(&app, &links.seats[2]).await.0, StatusCode::OK);
    ok_step(
        &app,
        &links.seats[0],
        json!({ "champion": "anarki", "game": 1 }),
    )
    .await;
    // O link é a credencial: quem abre o do colega, antes de entrar, é do lado.
    assert!(without_the_grid(&detail(&app, &links.seats[1]).await).contains("anarki"));
    assert!(!without_the_grid(&detail(&app, &links.seats[3]).await).contains("anarki"));
}

#[tokio::test]
async fn the_blind_duel_uses_the_window_in_the_room_too() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let links = links_of(&app, picks_only(1, 3, "blind")).await;
    let first = started(&app, &links).await;
    assert_eq!(first["veto"]["turn"]["action"], "team_pick");
    assert_eq!(first["veto"]["turn"]["sides"], json!(["a", "b"]));
    assert_eq!(first["veto"]["picks"].as_array().unwrap().len(), 3);
    ok_step(
        &app,
        &links.seats[0],
        json!({ "champion": "anarki", "game": 2 }),
    )
    .await;
    let other = detail(&app, &links.seats[1]).await;
    assert!(!without_the_grid(&other).contains("anarki"));
    assert_eq!(other["veto"]["games"][1]["pick_a"], Value::Null);
    // O fim do prazo sorteia o que falta e abre tudo.
    age(&db, 63).await;
    let done = detail(&app, &links.spectator).await;
    assert_eq!(done["veto"]["status"], "done");
    assert!(without_the_grid(&done).contains("anarki"));
    for game in done["veto"]["games"].as_array().unwrap() {
        assert_eq!(game["team_picks"].as_array().unwrap().len(), 2);
    }
}

#[tokio::test]
async fn blind_needs_a_mirror_and_the_open_window_is_not_chosen() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let no_mirror = json!({
        "team_size": 2,
        "best_of": 1,
        "veto": { "pick_order": "blind" },
        "champion_rules": { "mirror": false }
    });
    assert_eq!(
        create_room(&app, None, no_mirror).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("tournament.blind_needs_mirror")
        )
    );
    assert_eq!(
        create_room(&app, None, picks_only(2, 1, "open")).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("tournament.pick_order_invalid")
        )
    );
    // O padrão é o alternado, com o espelho desligado também.
    let links = links_of(
        &app,
        json!({ "team_size": 2, "best_of": 1, "champion_rules": { "mirror": false } }),
    )
    .await;
    assert_eq!(
        detail(&app, &links.manage).await["veto"]["pick_order"],
        "alternating"
    );
}

#[tokio::test]
async fn side_a_starts_when_asked() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    // Com veto de mapas, é o primeiro passo dele; muitas vezes, para o sorteio
    // não acertar por acaso.
    for _ in 0..8 {
        let links = links_of(&app, with(room(1, 3), json!({ "who_starts": "a" }))).await;
        let first = started(&app, &links).await;
        assert_eq!(first["veto"]["turn"]["target"], "map");
        assert_eq!(first["veto"]["turn"]["sides"], json!(["a"]));
    }
    // Sem veto de mapas, com bans, o primeiro passo é o ban: também dele.
    let links = links_of(
        &app,
        json!({
            "team_size": 1,
            "best_of": 3,
            "who_starts": "a",
            "veto": { "map_mode": "off" }
        }),
    )
    .await;
    let first = started(&app, &links).await;
    assert_eq!(first["veto"]["turn"]["action"], "ban");
    assert_eq!(first["veto"]["turn"]["sides"], json!(["a"]));
    // Sem bans, o primeiro pick.
    let links = links_of(
        &app,
        with(
            picks_only(2, 1, "alternating"),
            json!({ "who_starts": "a" }),
        ),
    )
    .await;
    let first = started(&app, &links).await;
    assert_eq!(first["veto"]["turn"]["sides"], json!(["a"]));
}

#[tokio::test]
async fn the_draw_is_still_the_default_and_the_sides_stay_fixed() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let mut starters = Vec::new();
    for _ in 0..24 {
        let links = links_of(&app, room(1, 1)).await;
        let first = started(&app, &links).await;
        starters.push(
            first["veto"]["turn"]["sides"][0]
                .as_str()
                .unwrap()
                .to_owned(),
        );
    }
    // Uma chance em 2^23 de falhar por azar.
    assert!(starters.iter().any(|s| s == "a"));
    assert!(starters.iter().any(|s| s == "b"));
}
