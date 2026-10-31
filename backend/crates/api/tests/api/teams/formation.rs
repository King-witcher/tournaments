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
    new_team(&app, &ana, &p[0]).await;
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
async fn the_organizer_moves_players_between_teams_and_the_reserve() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 3).await;
    create_tournament(&app, &ana, tdm(2, json!({}))).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&p[0], &p[1], &p[2]]).await;

    let (status, f) = new_team(&app, &ana, &p[0]).await;
    assert_eq!(status, StatusCode::OK, "{f}");
    let team = team_of(&f, &p[0]).unwrap()["id"].as_i64().unwrap();
    let (_, f) = move_to(&app, &ana, team, &p[1]).await;
    assert_eq!(rosters(&f), [vec![p[0].id, p[1].id]]);
    assert_eq!(reserves(&f), [p[2].id]);
    // Cheio.
    let (status, body) = move_to(&app, &ana, team, &p[2]).await;
    assert_eq!(
        (status, body),
        (
            StatusCode::CONFLICT,
            json!({ "error": "team.full", "params": { "max": 2 } })
        )
    );
    // Repetir não muda nada.
    assert_eq!(move_to(&app, &ana, team, &p[1]).await.0, StatusCode::OK);

    // O capitão vai para a reserva: a capitania passa para P2.
    let (status, f) = to_reserve(&app, &ana, team, &p[0]).await;
    assert_eq!(status, StatusCode::OK, "{f}");
    assert_eq!(team_of(&f, &p[1]).unwrap()["captain"], p[1].id);
    // P2 vai para um time novo: o antigo, vazio, some.
    let (_, f) = new_team(&app, &ana, &p[1]).await;
    assert_eq!(rosters(&f).len(), 1);
    assert_ne!(team_of(&f, &p[1]).unwrap()["id"], team);
    assert_eq!(
        move_to(&app, &ana, team, &p[2]).await,
        (StatusCode::NOT_FOUND, err("team.not_found"))
    );

    // Só a organização; quem não está inscrito, não.
    assert_eq!(new_team(&app, &bia, &p[2]).await.0, StatusCode::FORBIDDEN);
    assert_eq!(
        new_team(&app, &ana, &bia).await,
        (StatusCode::NOT_FOUND, err("participant.not_found"))
    );
    // Quem vê o torneio vê os times.
    assert_eq!(
        get(&app, &format!("{COPA}/teams"), &bia).await.0,
        StatusCode::OK
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
    let body = tdm(2, json!({ "check_in_minutes": 30 }));
    formed(&app, &ana, body, 2, &[&p[0], &p[1], &p[2], &p[3]]).await;
    register(&app, "copa", &[&p[4]]).await;
    starts_in(&db, "copa", 10).await;
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
    let team = team_of(&formation(&app, &ana).await, &p[3]).unwrap()["id"]
        .as_i64()
        .unwrap();
    to_reserve(&app, &ana, team, &p[3]).await;
    let (status, f) = move_to(&app, &ana, team, &p[4]).await;
    assert_eq!(status, StatusCode::OK, "{f}");
    let (status, body) = post_empty(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // Depois de começar, a formação não muda.
    assert_eq!(
        new_team(&app, &ana, &p[3]).await,
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
