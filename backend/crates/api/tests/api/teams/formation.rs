//! A formação dos times do TDM antes do início.

use super::*;
use crate::matches::players;

#[tokio::test]
async fn switching_the_mode_with_registrants_redoes_the_teams() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&p[0], &p[1], &p[2], &p[3]]).await;
    assert_eq!(
        formation(&app, &ana).await["teams"]
            .as_array()
            .unwrap()
            .len(),
        4
    );

    // Duelo → TDM: os times de um somem, todos ficam sem time.
    let (status, t) = patch(&app, COPA, &ana, json!({ "mode": "tdm" })).await;
    assert_eq!(status, StatusCode::OK, "{t}");
    assert_eq!(
        (t["mode"].clone(), t["team_size"].clone()),
        (json!("tdm"), json!(2))
    );
    let f = formation(&app, &ana).await;
    assert!(f["teams"].as_array().unwrap().is_empty());
    assert_eq!(reserves(&f), p.iter().map(|u| u.id).collect::<Vec<_>>());

    // Um time formado some ao mudar o tamanho.
    save(&app, &ana, json!([new_team(&[&p[0]])])).await;
    let (status, _) = patch(&app, COPA, &ana, json!({ "team_size": 3 })).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        formation(&app, &ana).await["teams"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    // TDM → duelo: cada inscrito volta a ter o time de um.
    let (status, t) = patch(&app, COPA, &ana, json!({ "mode": "duel" })).await;
    assert_eq!(status, StatusCode::OK, "{t}");
    assert_eq!(t["team_size"], 1);
    let f = formation(&app, &ana).await;
    assert_eq!(rosters(&f).len(), 4);
    assert!(reserves(&f).is_empty());

    // Tamanho fora do modo.
    let (status, body) = patch(&app, COPA, &ana, json!({ "team_size": 2 })).await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({ "error": "tournament.team_size_invalid", "params": { "min": 1, "max": 1 } })
        )
    );
}

#[tokio::test]
async fn slots_are_players_and_hold_two_teams() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let (status, body) = post(
        &app,
        "/tenants/quake-br/tournaments",
        &ana,
        tdm(3, json!({ "max_participants": 5 })),
    )
    .await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "error": "tournament.max_participants_invalid",
                "params": { "min": 6, "max": 1024 }
            })
        )
    );
    // 4v4 com 3 bans e sem repetir no time: 6 bans + 4 escolhas na partida.
    let rules =
        json!({ "bans_per_series": 3, "mirror": true, "same_team": false, "no_repeat": null });
    let body = tdm(4, json!({ "champion_rules": rules }));
    let created = create_tournament(&app, &ana, body).await;
    assert_eq!(
        (created["mode"].clone(), created["team_size"].clone()),
        (json!("tdm"), json!(4))
    );
}

#[tokio::test]
async fn random_draw_leaves_the_last_registered_as_reserve() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 5).await;
    create_tournament(&app, &ana, tdm(2, json!({}))).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&p[0], &p[1], &p[2], &p[3], &p[4]]).await;

    let (status, f) = draw(&app, &ana, "random").await;
    assert_eq!(status, StatusCode::OK, "{f}");
    let teams = rosters(&f);
    assert_eq!(teams.len(), 2);
    assert!(teams.iter().all(|t| t.len() == 2));
    assert_eq!(reserves(&f), [p[4].id]);
    // O capitão vem primeiro em cada time.
    for t in f["teams"].as_array().unwrap() {
        assert_eq!(t["captain"], t["members"][0]["user_id"]);
    }
    let seed: Option<i64> = sqlx::query_scalar("select teams_seed from tournaments")
        .fetch_one(&db)
        .await
        .unwrap();
    assert!(seed.is_some());

    // Só a organização, só no TDM.
    assert_eq!(draw(&app, &bia, "random").await.0, StatusCode::FORBIDDEN);
    let (status, body) = draw(&app, &ana, "shuffled").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}

#[tokio::test]
async fn balanced_draw_snakes_by_the_community_rating() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    // 1900, 1700, 1600 e 1400 (o inicial é 1500).
    for (who, delta) in [(&p[0], 400), (&p[1], 200), (&p[2], 100), (&p[3], -100)] {
        let uri = format!("/tenants/quake-br/ratings/{}/adjustments", who.id);
        let body = json!({ "delta": delta, "reason": "teste" });
        let (status, body) = post(&app, &uri, &ana, body).await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
    }
    create_tournament(&app, &ana, tdm(2, json!({}))).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&p[3], &p[2], &p[1], &p[0]]).await;

    let (status, f) = draw(&app, &ana, "balanced").await;
    assert_eq!(status, StatusCode::OK, "{f}");
    // 1º e 4º contra 2º e 3º; o capitão é o de maior rating.
    let mut teams = rosters(&f);
    teams.sort();
    let mut want = vec![vec![p[0].id, p[3].id], vec![p[1].id, p[2].id]];
    want.sort();
    assert_eq!(teams, want);
    let captain = team_of(&f, &p[3]).unwrap()["captain"].clone();
    assert_eq!(captain, p[0].id);
}

#[tokio::test]
async fn the_organizer_saves_the_whole_formation() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 5).await;
    create_tournament(&app, &ana, tdm(2, json!({}))).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&p[0], &p[1], &p[2], &p[3], &p[4]]).await;

    // Dois times novos; o resto, reserva. Time incompleto passa.
    let teams = json!([new_team(&[&p[0], &p[1]]), new_team(&[&p[2]])]);
    let (status, f) = save(&app, &ana, teams).await;
    assert_eq!(status, StatusCode::OK, "{f}");
    assert_eq!(rosters(&f), [vec![p[0].id, p[1].id], vec![p[2].id]]);
    assert_eq!(reserves(&f), [p[3].id, p[4].id]);
    assert_eq!(
        f["registration_order"],
        json!(p.iter().map(|u| u.id).collect::<Vec<_>>())
    );

    // O time que fica guarda o id e o nome; o capitão enviado vale, mesmo
    // saindo o antigo. O que não vem some, e quem não aparece fica de reserva.
    let (first, second) = (
        team_of(&f, &p[0]).unwrap()["id"].as_i64().unwrap(),
        team_of(&f, &p[2]).unwrap()["id"].as_i64().unwrap(),
    );
    let uri = format!("{COPA}/teams/{first}");
    let (status, body) = patch(&app, &uri, &ana, json!({ "name": "Os Bons" })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let teams = json!([{ "id": first, "members": [p[1].id, p[3].id], "captain": p[3].id }]);
    let (status, f) = save(&app, &ana, teams).await;
    assert_eq!(status, StatusCode::OK, "{f}");
    assert_eq!(rosters(&f), [vec![p[3].id, p[1].id]]);
    assert_eq!(f["teams"][0]["name"], "Os Bons");
    assert_eq!(f["teams"][0]["captain"], p[3].id);
    assert_eq!(reserves(&f), [p[0].id, p[2].id, p[4].id]);
    let (status, body) = save(
        &app,
        &ana,
        json!([{ "id": second, "members": [p[2].id], "captain": p[2].id }]),
    )
    .await;
    assert_eq!(
        (status, body),
        (StatusCode::NOT_FOUND, err("team.not_found"))
    );

    // Time vazio some, com ou sem id.
    let (status, f) = save(&app, &ana, json!([{ "id": first, "members": [] }])).await;
    assert_eq!(status, StatusCode::OK, "{f}");
    assert!(rosters(&f).is_empty());
    let (status, f) = save(&app, &ana, json!([{ "members": [] }])).await;
    assert_eq!(status, StatusCode::OK, "{f}");
    assert_eq!(reserves(&f).len(), 5);

    // Só a organização; quem vê o torneio vê os times.
    assert_eq!(save(&app, &bia, json!([])).await.0, StatusCode::FORBIDDEN);
    assert_eq!(
        get(&app, &format!("{COPA}/teams"), &bia).await.0,
        StatusCode::OK
    );
}

#[tokio::test]
async fn the_saved_formation_keeps_the_captains_it_was_sent() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    let teams = formed(
        &app,
        &ana,
        tdm(2, json!({})),
        2,
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    let order = json!({ "team_ids": [teams[1], teams[0]] });
    let (status, body) = put(&app, &format!("{COPA}/seeding"), &ana, order).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // Os capitães trocam de time um com o outro (o índice é um capitão por
    // torneio, e a troca passa por ele); cada time segue com o seed dele.
    let body = json!([
        { "id": teams[0], "members": [p[2].id, p[1].id], "captain": p[2].id },
        { "id": teams[1], "members": [p[0].id, p[3].id], "captain": p[0].id },
    ]);
    let (status, f) = save(&app, &ana, body).await;
    assert_eq!(status, StatusCode::OK, "{f}");
    let (first, second) = (team_of(&f, &p[2]).unwrap(), team_of(&f, &p[0]).unwrap());
    assert_eq!(
        (
            first["id"].clone(),
            first["captain"].clone(),
            first["seed"].clone()
        ),
        (json!(teams[0]), json!(p[2].id), json!(2))
    );
    assert_eq!(
        (
            second["id"].clone(),
            second["captain"].clone(),
            second["seed"].clone()
        ),
        (json!(teams[1]), json!(p[0].id), json!(1))
    );

    // O capitão de um time existente vai ser capitão de um time novo.
    let body = json!([
        { "id": teams[1], "members": [p[3].id], "captain": p[3].id },
        { "members": [p[0].id], "captain": p[0].id },
    ]);
    let (status, f) = save(&app, &ana, body).await;
    assert_eq!(status, StatusCode::OK, "{f}");
    assert_eq!(team_of(&f, &p[3]).unwrap()["id"], teams[1]);
    assert_eq!(team_of(&f, &p[0]).unwrap()["captain"], p[0].id);
}

#[tokio::test]
async fn the_saved_formation_is_validated() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 3).await;
    let teams = formed(&app, &ana, tdm(2, json!({})), 2, &[&p[0], &p[1]]).await;
    register(&app, "copa", &[&p[2]]).await;
    let team = |members: &[&User], captain: &User| {
        let ids: Vec<i64> = members.iter().map(|u| u.id).collect();
        json!({ "id": teams[0], "members": ids, "captain": captain.id })
    };

    assert_eq!(
        save(&app, &ana, json!([team(&[&p[0], &p[1], &p[2]], &p[0])])).await,
        (
            StatusCode::CONFLICT,
            json!({ "error": "team.full", "params": { "max": 2 } })
        )
    );
    assert_eq!(
        save(&app, &ana, json!([team(&[&p[0], &bia], &p[0])])).await,
        (StatusCode::NOT_FOUND, err("participant.not_found"))
    );
    assert_eq!(
        save(&app, &ana, json!([team(&[&p[0], &p[0]], &p[0])])).await,
        (StatusCode::CONFLICT, err("team.player_in_team"))
    );
    assert_eq!(
        save(&app, &ana, json!([team(&[&p[0]], &p[2])])).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("team.captain_invalid")
        )
    );
    assert_eq!(
        save(
            &app,
            &ana,
            json!([{ "id": teams[0] + 1000, "members": [p[0].id], "captain": p[0].id }])
        )
        .await,
        (StatusCode::NOT_FOUND, err("team.not_found"))
    );
    // Nada mudou.
    let f = formation(&app, &ana).await;
    assert_eq!(rosters(&f), [vec![p[0].id, p[1].id]]);
    assert_eq!(reserves(&f), [p[2].id]);
}

#[tokio::test]
async fn the_saved_formation_warns_the_realtime() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    create_tournament(&app, &ana, tdm(2, json!({}))).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&p[0], &p[1]]).await;
    crate::realtime::clear(&db).await;

    let (status, f) = save(&app, &ana, json!([new_team(&[&p[0], &p[1]])])).await;
    assert_eq!(status, StatusCode::OK, "{f}");
    assert!(
        crate::realtime::outbox(&db)
            .await
            .contains(&crate::realtime::tournament("copa", None))
    );
}

#[tokio::test]
async fn a_captain_who_cancels_hands_the_team_over() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    formed(&app, &ana, tdm(2, json!({})), 2, &[&p[0], &p[1]]).await;
    let (status, body) = delete(&app, &format!("{COPA}/registration"), &p[0]).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    let f = formation(&app, &ana).await;
    assert_eq!(rosters(&f), [vec![p[1].id]]);
    assert_eq!(f["teams"][0]["captain"], p[1].id);
}

#[tokio::test]
async fn the_start_needs_every_team_complete() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 5).await;
    let body = tdm(2, json!({ "check_in": true }));
    formed(&app, &ana, body, 2, &[&p[0], &p[1], &p[2], &p[3]]).await;
    register(&app, "copa", &[&p[4]]).await;
    open_check_in(&app, &ana, "copa").await;
    for who in &p {
        let (status, body) = post_empty(&app, &format!("{COPA}/check-in"), who).await;
        assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    }
    // P4 desfaz o check-in: o time dele fica incompleto. P5 (reserva) não
    // atrapalha.
    let (status, _) = delete(&app, &format!("{COPA}/check-in"), &p[3]).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, body) = post_empty(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(
        (status, body),
        (
            StatusCode::CONFLICT,
            json!({ "error": "tournament.teams_incomplete", "params": { "teams": 1 } })
        )
    );
    // O seeding só tem o time completo.
    let (_, seeding) = get(&app, &format!("{COPA}/seeding"), &ana).await;
    assert_eq!(seeding.as_array().unwrap().len(), 1);

    // A organização troca P4 pelo reserva: dois times completos.
    let team = team_of(&formation(&app, &ana).await, &p[3]).unwrap();
    let other = team_of(&formation(&app, &ana).await, &p[0]).unwrap();
    let body = json!([
        { "id": other["id"], "members": [p[0].id, p[1].id], "captain": p[0].id },
        { "id": team["id"], "members": [p[2].id, p[4].id], "captain": p[2].id },
    ]);
    let (status, f) = save(&app, &ana, body).await;
    assert_eq!(status, StatusCode::OK, "{f}");
    let (status, body) = post_empty(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // Depois de começar, a formação não muda.
    assert_eq!(
        save(&app, &ana, json!([new_team(&[&p[3]])])).await,
        (StatusCode::CONFLICT, err("tournament.participants_locked"))
    );
    assert_eq!(
        draw(&app, &ana, "random").await,
        (StatusCode::CONFLICT, err("tournament.participants_locked"))
    );
}

#[tokio::test]
async fn one_team_is_not_enough() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 3).await;
    formed(&app, &ana, tdm(2, json!({})), 2, &[&p[0], &p[1]]).await;
    register(&app, "copa", &[&p[2]]).await;
    let (status, body) = post_empty(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(
        (status, body),
        (
            StatusCode::CONFLICT,
            json!({ "error": "tournament.not_enough_eligible", "params": { "min": 2, "eligible": 1 } })
        )
    );
}
