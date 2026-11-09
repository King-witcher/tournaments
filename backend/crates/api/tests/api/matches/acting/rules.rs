//! O que não cobra quem não pode agir: tarefas e avisos. A regra no torneio e o duelo.

use super::*;

// Tarefas e avisos.

#[tokio::test]
async fn the_bracket_and_my_tournaments_do_not_charge_whoever_cannot_act() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cast(&db, all_captain(), draft("alternating", 1, 1)).await;
    let [cap, mate] = [&c.p[0], &c.p[1]];
    let task_in_bracket = |b: &Value| find(b, KEY)["task"].clone();
    let next_task = async |who: &User| {
        let (status, body) = get(&c.app, "/me/tournaments", who).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        body["registered"][0]["next_match"]["task"].clone()
    };
    assert_eq!(task_in_bracket(&bracket(&c.app, cap).await), "ready");
    assert_eq!(
        task_in_bracket(&bracket(&c.app, mate).await),
        "await_captain"
    );
    assert_eq!(next_task(cap).await, "ready");
    assert_eq!(next_task(mate).await, "await_captain");
    // Na vez do lado dele, no veto, de novo só o capitão é cobrado.
    captains_ready(&c).await;
    let [other_cap, other_mate] = [&c.p[2], &c.p[3]];
    assert_eq!(next_task(other_cap).await, "veto");
    assert_eq!(next_task(other_mate).await, "await_captain");
    assert_eq!(next_task(cap).await, "await_opponent");
    assert_eq!(next_task(mate).await, "await_opponent");
}

#[tokio::test]
async fn what_asks_for_action_goes_to_who_can_act_and_the_rest_to_the_team() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    let body = tdm(2, with(all_captain(), draft("alternating", 0, 1)));
    started(&app, &ana, body, 2, &[&p[0], &p[1], &p[2], &p[3]]).await;
    let to = |events: &[(String, Vec<i64>, Value)], code: &str| -> Vec<Vec<i64>> {
        events
            .iter()
            .filter(|(c, _, _)| c == code)
            .map(|(_, to, _)| to.clone())
            .collect()
    };
    // A série pronta só informa: vai para o time.
    let events = notices(&outbox(&db).await);
    assert_eq!(
        to(&events, "match.ready"),
        [ids(&[&p[0], &p[1]]), ids(&[&p[2], &p[3]])]
    );
    clear(&db).await;
    post_empty(&app, &uri("ready"), &p[0]).await;
    clear(&db).await;
    let (status, body) = post_empty(&app, &uri("ready"), &p[2]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let events = notices(&outbox(&db).await);
    // Veto e picks só com o capitão: só ele é chamado.
    assert_eq!(
        to(&events, "match.veto_started"),
        [vec![p[0].id], vec![p[2].id]]
    );
    assert_eq!(to(&events, "match.team_pick_open"), [ids(&[&p[0], &p[2]])]);
}

#[tokio::test]
async fn with_each_player_picking_the_opening_of_the_picks_goes_to_everyone() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    // Mapa e ban no capitão; os picks, de cada um.
    let mix = rules("captain", "captain", "each_player", "captain");
    let body = tdm(2, with(mix, draft("alternating", 0, 1)));
    started(&app, &ana, body, 2, &[&p[0], &p[1], &p[2], &p[3]]).await;
    for who in &p[..3] {
        post_empty(&app, &uri("ready"), who).await;
    }
    clear(&db).await;
    let (status, body) = post_empty(&app, &uri("ready"), &p[3]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let events = notices(&outbox(&db).await);
    let code = |wanted: &str| -> Vec<Vec<i64>> {
        events
            .iter()
            .filter(|(c, _, _)| c == wanted)
            .map(|(_, to, _)| to.clone())
            .collect()
    };
    assert_eq!(
        code("match.team_pick_open"),
        [ids(&[&p[0], &p[1], &p[2], &p[3]])]
    );
    assert_eq!(
        code("match.veto_started"),
        [ids(&[&p[0], &p[1]]), ids(&[&p[2], &p[3]])]
    );
}

// A regra no torneio.

#[tokio::test]
async fn the_rules_are_saved_and_lock_when_the_tournament_starts() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    // Dita na criação, vale a dita.
    let created = create_tournament(&app, &ana, tdm(2, json!({}))).await;
    assert_eq!(
        created["team_rules"],
        json!({ "veto": "anyone", "ban": "anyone", "pick": "each_player", "report": "anyone" })
    );
    let (status, body) = patch(&app, COPA, &ana, all_captain()).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["team_rules"]["report"], "captain");
    // Cada coluna com os valores dela.
    for (column, constraint) in [
        ("veto_by", "tournaments_veto_by"),
        ("ban_by", "tournaments_ban_by"),
        ("pick_by", "tournaments_pick_by"),
        ("report_by", "tournaments_report_by"),
    ] {
        assert_eq!(
            rejected(exec(&db, &format!("update tournaments set {column} = 'x'")).await),
            rule(CHECK, constraint),
            "{column}"
        );
    }
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&p[0], &p[1], &p[2], &p[3]]).await;
    let teams = json!([
        crate::teams::new_team(&[&p[0], &p[1]]),
        crate::teams::new_team(&[&p[2], &p[3]]),
    ]);
    let (status, f) = crate::teams::save(&app, &ana, teams).await;
    assert_eq!(status, StatusCode::OK, "{f}");
    let (status, body) = post_empty(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // O mesmo não é mudança, e qualquer outra, inclusive a de resultado, é recusada.
    let (status, body) = patch(&app, COPA, &ana, all_captain()).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let changed = rules("captain", "captain", "captain", "anyone");
    assert_eq!(
        patch(&app, COPA, &ana, changed).await,
        (StatusCode::CONFLICT, err("tournament.setup_locked"))
    );
    // E o banco também recusa.
    for column in [
        "veto_by = 'anyone'",
        "ban_by = 'anyone'",
        "pick_by = 'each_player'",
        "report_by = 'anyone'",
    ] {
        assert_eq!(
            rejected(exec(&db, &format!("update tournaments set {column}")).await),
            rule(CHECK, "tournaments_veto_started_frozen"),
            "{column}"
        );
    }
}

#[tokio::test]
async fn a_duel_is_the_same_whatever_the_rules_say() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    let body = copa(with(all_captain(), draft("alternating", 1, 1)));
    start(&app, &ana, body, &[&p[0], &p[1]]).await;
    assert_eq!(detail(&app, &p[0], KEY).await["summary"]["task"], "ready");
    for who in &p {
        let (status, body) = post_empty(&app, &uri("ready"), who).await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    let seen = detail(&app, &p[1], KEY).await;
    assert_eq!(seen["veto"]["captains"], json!([]));
    assert_eq!(seen["summary"]["task"], "veto");
    let (status, body) = post(&app, &uri("steps"), &p[1], json!({ "champion": "anarki" })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = post(&app, &uri("steps"), &p[0], json!({ "champion": "athena" })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = post(&app, &uri("steps"), &p[0], json!({ "champion": "clutch" })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = post(
        &app,
        &uri("steps"),
        &p[1],
        json!({ "champion": "doom-slayer" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["veto"]["status"], "done");
    let (status, body) = report(&app, &p[0], KEY, 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = confirm(&app, &p[1], KEY, 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}
