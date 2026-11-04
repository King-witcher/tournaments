//! Os passos do veto × a regra: vetar, votar, banir, escolher (o capitão por um
//! colega; às cegas, sem vazar), pedir e responder o desfazer, o organizador e o
//! capitão ausente.

use super::*;

// Vetar mapas.

async fn map_veto_by(team_rule: &str, colleague_acts: bool) {
    let (db, _lease) = crate::db::test_db().await;
    let c = cast(
        &db,
        rules(team_rule, "anyone", "each_player", "anyone"),
        map_veto(),
    )
    .await;
    all_ready(&c).await;
    let ban = json!({ "map": "awoken" });
    let [cap, mate, other_cap, other_mate] = [&c.p[0], &c.p[1], &c.p[2], &c.p[3]];
    let forbidden = StatusCode::FORBIDDEN;
    // O adversário não é da vez; o colega dele, pela regra, nem chega a isso.
    refused(
        &c,
        other_cap,
        ban.clone(),
        StatusCode::CONFLICT,
        NOT_YOUR_TURN,
    )
    .await;
    if colleague_acts {
        let conflict = StatusCode::CONFLICT;
        refused(&c, other_mate, ban.clone(), conflict, NOT_YOUR_TURN).await;
    } else {
        refused(&c, other_mate, ban.clone(), forbidden, NOT_CAPTAIN).await;
    }
    // Quem não joga, mesmo organizando, não veta.
    refused(&c, &c.ana, ban.clone(), forbidden, NOT_PLAYER).await;
    refused(&c, &c.bia, ban.clone(), forbidden, NOT_PLAYER).await;
    let seen = detail(&c.app, mate, KEY).await;
    assert_eq!(seen["veto"]["can_act"]["map"], colleague_acts);
    let by = if colleague_acts {
        assert!(!seen["veto"]["options"].as_array().unwrap().is_empty());
        assert_eq!(seen["summary"]["task"], "veto");
        mate
    } else {
        assert!(seen["veto"]["options"].as_array().unwrap().is_empty());
        assert_eq!(seen["summary"]["task"], "await_captain");
        refused(&c, mate, ban.clone(), forbidden, NOT_CAPTAIN).await;
        cap
    };
    let done = ok_step(&c, by, ban).await;
    assert_eq!(last_event(&done)["actor"], by_name(&c, by));
}

fn by_name(c: &Cast, who: &User) -> Value {
    let i = c.p.iter().position(|u| u.id == who.id).unwrap();
    json!(format!("P{}", i + 1))
}

#[tokio::test]
async fn vetoing_a_map_by_the_captain() {
    map_veto_by("captain", false).await;
}

#[tokio::test]
async fn vetoing_a_map_by_anyone() {
    map_veto_by("anyone", true).await;
}

// Votar.

async fn vote_by(team_rule: &str, colleague_acts: bool) {
    let (db, _lease) = crate::db::test_db().await;
    let c = cast(
        &db,
        rules(team_rule, "anyone", "each_player", "anyone"),
        json!({ "veto": { "map_mode": "vote", "champion_phase": false } }),
    )
    .await;
    all_ready(&c).await;
    let seen = detail(&c.app, &c.p[0], KEY).await;
    let offered = seen["veto"]["games"][0]["offered"].clone();
    let vote = json!({ "map": offered[0]["slug"] });
    let [cap, mate, other_cap] = [&c.p[0], &c.p[1], &c.p[2]];
    refused(&c, &c.ana, vote.clone(), StatusCode::FORBIDDEN, NOT_PLAYER).await;
    if colleague_acts {
        ok_step(&c, mate, vote.clone()).await;
    } else {
        let seen = detail(&c.app, mate, KEY).await;
        assert_eq!(seen["summary"]["task"], "await_captain");
        refused(&c, mate, vote.clone(), StatusCode::FORBIDDEN, NOT_CAPTAIN).await;
        ok_step(&c, cap, vote.clone()).await;
    }
    // O adversário vota o dele.
    let done = ok_step(&c, other_cap, vote).await;
    assert_eq!(done["veto"]["status"], "done");
}

#[tokio::test]
async fn voting_by_the_captain() {
    vote_by("captain", false).await;
}

#[tokio::test]
async fn voting_by_anyone() {
    vote_by("anyone", true).await;
}

// Banir.

async fn ban_by(team_rule: &str, colleague_acts: bool) {
    let (db, _lease) = crate::db::test_db().await;
    let c = cast(
        &db,
        rules("anyone", team_rule, "each_player", "anyone"),
        draft("alternating", 1, 1),
    )
    .await;
    all_ready(&c).await;
    let ban = json!({ "champion": "anarki" });
    // O primeiro ban é do seed pior, B.
    let [other_cap, cap, mate] = [&c.p[0], &c.p[2], &c.p[3]];
    refused(
        &c,
        other_cap,
        ban.clone(),
        StatusCode::CONFLICT,
        NOT_YOUR_TURN,
    )
    .await;
    refused(&c, &c.ana, ban.clone(), StatusCode::FORBIDDEN, NOT_PLAYER).await;
    let seen = detail(&c.app, mate, KEY).await;
    assert_eq!(seen["veto"]["can_act"]["ban"], colleague_acts);
    let by = if colleague_acts {
        mate
    } else {
        assert!(seen["veto"]["options"].as_array().unwrap().is_empty());
        refused(&c, mate, ban.clone(), StatusCode::FORBIDDEN, NOT_CAPTAIN).await;
        cap
    };
    ok_step(&c, by, ban).await;
}

#[tokio::test]
async fn banning_by_the_captain() {
    ban_by("captain", false).await;
}

#[tokio::test]
async fn banning_by_anyone() {
    ban_by("anyone", true).await;
}

// Escolher.

#[tokio::test]
async fn the_captain_picks_for_each_teammate_and_is_the_actor() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cast(&db, all_captain(), draft("alternating", 0, 1)).await;
    captains_ready(&c).await;
    let [cap, mate, other_cap, other_mate] = [&c.p[0], &c.p[1], &c.p[2], &c.p[3]];

    // O capitão tem um pick por colega que falta, o próximo primeiro; o colega, nenhum.
    let seen = detail(&c.app, cap, KEY).await;
    assert_eq!(seen["veto"]["can_act"]["pick_for_team"], true);
    assert_eq!(seen["summary"]["task"], "veto");
    let slots = seen["veto"]["picks"].as_array().unwrap();
    assert_eq!(slots.len(), 2);
    assert_eq!(slots[0]["player"], cap.id);
    assert_eq!(slots[1]["player"], mate.id);
    assert_eq!(slots[1]["nickname"], "P2");
    let seen = detail(&c.app, mate, KEY).await;
    assert_eq!(seen["veto"]["can_act"]["pick"], false);
    assert_eq!(seen["veto"]["picks"], json!([]));
    assert_eq!(seen["summary"]["task"], "await_captain");
    assert_eq!(seen["veto"]["captains"], json!([cap.id, other_cap.id]));

    // Recusados: o colega, mesmo para si; quem não joga; o adversário fora da vez.
    let pick = |player: &User| json!({ "champion": "anarki", "player": player.id });
    let forbidden = StatusCode::FORBIDDEN;
    refused(&c, mate, pick(mate), forbidden, NOT_CAPTAIN).await;
    refused(&c, &c.ana, pick(mate), forbidden, NOT_PLAYER).await;
    refused(&c, other_mate, pick(other_mate), forbidden, NOT_CAPTAIN).await;
    refused(
        &c,
        other_cap,
        pick(other_cap),
        StatusCode::CONFLICT,
        NOT_YOUR_TURN,
    )
    .await;
    // Colega de outro lado não vale.
    refused(
        &c,
        cap,
        pick(other_mate),
        StatusCode::CONFLICT,
        NOT_YOUR_TURN,
    )
    .await;

    // Por um colega: o histórico grava os dois, e o campeão é dele.
    let done = ok_step(&c, cap, pick(mate)).await;
    let last = last_event(&done);
    assert_eq!(last["actor"], "P1");
    assert_eq!(last["player"], "P2");
    let picks = &done["veto"]["games"][0]["team_picks"];
    assert_eq!(picks[0]["user_id"], mate.id);

    // Sem dizer quem, vale o próximo sem escolha: B em O, X, X, O.
    let done = ok_step(&c, other_cap, json!({ "champion": "athena" })).await;
    assert_eq!(
        done["veto"]["games"][0]["team_picks"][1]["user_id"],
        other_cap.id
    );
    let done = ok_step(&c, other_cap, json!({ "champion": "clutch" })).await;
    assert_eq!(
        done["veto"]["games"][0]["team_picks"][2]["user_id"],
        other_mate.id
    );

    // Já escolheu: é o que sobra.
    refused(
        &c,
        cap,
        json!({ "champion": "doom-slayer", "player": mate.id }),
        StatusCode::CONFLICT,
        "veto.already_picked",
    )
    .await;
    let done = ok_step(&c, cap, json!({ "champion": "doom-slayer" })).await;
    assert_eq!(done["veto"]["games"][0]["team_picks"][3]["user_id"], cap.id);
    assert_eq!(done["veto"]["status"], "done");
}

#[tokio::test]
async fn each_player_picks_their_own_and_nobody_else() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cast(&db, all_players(), draft("alternating", 0, 1)).await;
    all_ready(&c).await;
    let [cap, mate, other_cap] = [&c.p[0], &c.p[1], &c.p[2]];
    let seen = detail(&c.app, cap, KEY).await;
    assert_eq!(seen["veto"]["can_act"]["pick_for_team"], false);
    let slots = seen["veto"]["picks"].as_array().unwrap();
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0]["player"], cap.id);
    // Nem o capitão escolhe pelo colega.
    refused(
        &c,
        cap,
        json!({ "champion": "anarki", "player": mate.id }),
        StatusCode::FORBIDDEN,
        "veto.not_your_pick",
    )
    .await;
    refused(
        &c,
        other_cap,
        json!({ "champion": "anarki" }),
        StatusCode::CONFLICT,
        NOT_YOUR_TURN,
    )
    .await;
    refused(
        &c,
        &c.ana,
        json!({ "champion": "anarki" }),
        StatusCode::FORBIDDEN,
        NOT_PLAYER,
    )
    .await;
    // Dizer a si mesmo é o mesmo que não dizer.
    let done = ok_step(&c, mate, json!({ "champion": "anarki", "player": mate.id })).await;
    let last = last_event(&done);
    assert_eq!(
        (&last["actor"], &last["player"]),
        (&json!("P2"), &json!("P2"))
    );
}

#[tokio::test]
async fn blind_with_the_captain_leaks_nothing_outside_the_side() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cast(&db, all_captain(), draft("blind", 0, 3)).await;
    captains_ready(&c).await;
    let [cap, mate, other_cap, other_mate] = [&c.p[0], &c.p[1], &c.p[2], &c.p[3]];
    let seen = detail(&c.app, cap, KEY).await;
    // Dois jogadores, três partidas, na ordem das partidas.
    let slots = seen["veto"]["picks"].as_array().unwrap();
    assert_eq!(slots.len(), 6);
    let order: Vec<(i64, i64)> = slots
        .iter()
        .map(|s| (s["game"].as_i64().unwrap(), s["player"].as_i64().unwrap()))
        .collect();
    assert_eq!(order[..3], [(1, cap.id), (1, mate.id), (2, cap.id)]);

    ok_step(
        &c,
        cap,
        json!({ "champion": "anarki", "game": 1, "player": mate.id }),
    )
    .await;
    let without_the_grid = |body: &Value| {
        let mut body = body.clone();
        let veto = body["veto"].as_object_mut().unwrap();
        veto.remove("champions");
        veto.remove("picks");
        veto.remove("options");
        body.to_string()
    };
    // O colega vê o do próprio time, sem poder escolher.
    let seen = detail(&c.app, mate, KEY).await;
    assert!(without_the_grid(&seen).contains("anarki"), "{seen}");
    assert_eq!(seen["veto"]["picks"], json!([]));
    // Quem é de fora, não: só que ele já escolheu.
    for who in [other_cap, other_mate, &c.ana, &c.bia] {
        let seen = detail(&c.app, who, KEY).await;
        assert!(!without_the_grid(&seen).contains("anarki"), "{seen}");
        assert_eq!(
            seen["veto"]["games"][0]["hidden_picks"][0]["user_id"],
            mate.id
        );
    }
    let bracket = bracket(&c.app, other_cap).await;
    assert!(!bracket.to_string().contains("anarki"));
    assert!(
        !outbox(&db)
            .await
            .iter()
            .any(|e| e.to_string().contains("anarki"))
    );
}

// Pedir e responder o desfazer.

#[tokio::test]
async fn undo_belongs_to_whoever_can_do_that_kind_of_step() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cast(
        &db,
        rules("anyone", "captain", "each_player", "anyone"),
        json!({
            "veto": { "map_mode": "veto", "champion_phase": true, "map_pool": ["awoken", "crucible"] },
            "champion_rules": {
                "bans_per_series": 1, "mirror": true, "same_team": false, "no_repeat": null
            },
        }),
    )
    .await;
    all_ready(&c).await;
    let [cap, mate, other_cap, other_mate] = [&c.p[0], &c.p[1], &c.p[2], &c.p[3]];
    let (request_uri, answer_uri) = (uri("undo-request"), uri("undo-request/answer"));
    let request = |who| post_empty(&c.app, &request_uri, who);
    let answer = |who, accept: bool| post(&c.app, &answer_uri, who, json!({ "accept": accept }));
    let not_captain = (StatusCode::FORBIDDEN, err(NOT_CAPTAIN));

    // O último passo é de mapa: a regra é de todos.
    ok_step(&c, mate, json!({ "map": "awoken" })).await;
    let seen = detail(&c.app, other_mate, KEY).await;
    assert_eq!(seen["veto"]["can_act"]["undo"], true);
    let (status, body) = request(other_mate).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = answer(mate, false).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // O último passo é de ban: a regra é do capitão, nos dois lados.
    ok_step(&c, other_cap, json!({ "champion": "anarki" })).await;
    let seen = detail(&c.app, mate, KEY).await;
    assert_eq!(seen["veto"]["can_undo"], true);
    assert_eq!(seen["veto"]["can_act"]["undo"], false);
    assert_eq!(request(mate).await, not_captain);
    assert_eq!(request(other_mate).await, not_captain);
    assert_eq!(
        request(&c.bia).await,
        (StatusCode::FORBIDDEN, err(NOT_PLAYER))
    );
    assert_eq!(
        request(&c.ana).await,
        (StatusCode::FORBIDDEN, err(NOT_PLAYER))
    );
    let (status, body) = request(other_cap).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(answer(mate, true).await, not_captain);
    assert_eq!(
        answer(other_cap, true).await,
        (StatusCode::FORBIDDEN, err("veto.own_request"))
    );
    assert_eq!(
        answer(&c.ana, true).await,
        (StatusCode::FORBIDDEN, err(NOT_PLAYER))
    );
    let (status, body) = answer(cap, true).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["veto"]["bans"], json!([]));

    // Desfeito o ban, o último volta a ser o de mapa.
    let seen = detail(&c.app, mate, KEY).await;
    assert_eq!(seen["veto"]["can_act"]["undo"], true);
}

// O organizador continua como está.

#[tokio::test]
async fn the_organizer_who_plays_as_a_teammate_keeps_the_organizer_actions_only() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 3).await;
    let org = organizer(&app, &db, &ana, "Org").await;
    let body = tdm(2, with(all_captain(), draft("alternating", 1, 3)));
    started(&app, &ana, body, 2, &[&p[0], &org, &p[1], &p[2]]).await;

    // Não marca "Pronto" pelo time, mas força o começo.
    let not_captain = (StatusCode::FORBIDDEN, err(NOT_CAPTAIN));
    assert_eq!(
        post_empty(&app, &uri("ready"), &org).await,
        not_captain.clone()
    );
    let (status, body) = post_empty(&app, &uri("begin"), &org).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["can_manage"], true);
    assert_eq!(body["can_report"], false);

    // O ban é do capitão de B; depois do de A, e o organizador não bane por A.
    let (status, body) = post(&app, &uri("steps"), &p[1], json!({ "champion": "anarki" })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        post(&app, &uri("steps"), &org, json!({ "champion": "athena" })).await,
        not_captain.clone()
    );
    // Desfaz direto.
    let (status, body) = post_empty(&app, &uri("undo"), &org).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["veto"]["bans"], json!([]));

    // Não envia como jogador, mas lança.
    assert_eq!(
        report(&app, &org, KEY, 1, game("a", "awoken")).await,
        not_captain
    );
    let (status, body) = set_game(&app, &org, KEY, 1, game("a", "awoken"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // E dá W.O.
    let (status, body) = set_result(&app, &org, KEY, walkover("a"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["summary"]["result"]["walkover"], true);
}

// Capitão ausente.

#[tokio::test]
async fn an_absent_captain_is_drawn_by_the_clock_and_can_be_replaced_during_the_tournament() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cast(&db, all_captain(), draft("alternating", 0, 1)).await;
    captains_ready(&c).await;
    let [cap, mate] = [&c.p[0], &c.p[1]];
    // O colega não pode; o organizador troca o capitão no meio do veto.
    let pick = json!({ "champion": "anarki" });
    refused(&c, mate, pick.clone(), StatusCode::FORBIDDEN, NOT_CAPTAIN).await;
    let team = format!("{COPA}/teams/{}", c.teams[0]);
    let (status, body) = patch(&c.app, &team, &c.ana, json!({ "captain": mate.id })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    refused(&c, cap, pick.clone(), StatusCode::FORBIDDEN, NOT_CAPTAIN).await;
    let done = ok_step(&c, mate, pick).await;
    assert_eq!(done["veto"]["captains"][0], mate.id);

    // Ninguém age: o prazo sorteia o que faltar.
    age_veto(&db, 33).await;
    let seen = detail(&c.app, cap, KEY).await;
    assert_eq!(seen["veto"]["games"][0]["team_picks"][1]["timeout"], true);
}

/// Volta `seconds` segundos no relógio de todos os eventos de veto.
async fn age_veto(db: &PgPool, seconds: i32) {
    for sql in [
        "alter table match_veto_events disable trigger match_veto_events_guard".to_owned(),
        format!(
            "update match_veto_events set created_at = created_at - interval '{seconds} seconds'"
        ),
        "alter table match_veto_events enable trigger match_veto_events_guard".to_owned(),
    ] {
        sqlx::query(sqlx::AssertSqlSafe(sql))
            .execute(db)
            .await
            .unwrap();
    }
}
