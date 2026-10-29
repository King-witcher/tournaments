//! O time depois de formado: nome, capitão, substituição e quem sai da
//! comunidade no meio do torneio.

use super::*;
use crate::matches::{confirm, detail, game, players, report, single};

fn team_uri(team: i64) -> String {
    format!("{COPA}/teams/{team}")
}

#[tokio::test]
async fn the_captain_names_the_team_and_the_organizer_picks_the_captain() {
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

    // O capitão (P1) dá o nome; o outro jogador (P2) não.
    let (status, f) = patch(
        &app,
        &team_uri(teams[0]),
        &p[0],
        json!({ "name": " Os Bons " }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{f}");
    assert_eq!(team_of(&f, &p[0]).unwrap()["name"], "Os Bons");
    assert_eq!(
        patch(&app, &team_uri(teams[0]), &p[1], json!({ "name": "X" })).await,
        (StatusCode::FORBIDDEN, err("team.not_captain"))
    );
    // O capitão não troca o capitão.
    assert_eq!(
        patch(
            &app,
            &team_uri(teams[0]),
            &p[0],
            json!({ "captain": p[1].id })
        )
        .await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
    // Único no torneio, sem diferenciar maiúsculas.
    assert_eq!(
        patch(
            &app,
            &team_uri(teams[1]),
            &p[2],
            json!({ "name": "os bons" })
        )
        .await,
        (StatusCode::CONFLICT, err("team.name_taken"))
    );
    let (status, body) = patch(
        &app,
        &team_uri(teams[1]),
        &p[2],
        json!({ "name": "a".repeat(41) }),
    )
    .await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({ "error": "team.name_invalid", "params": { "max": 40 } })
        )
    );

    // A organização troca o capitão, só por um jogador do time.
    let (status, f) = patch(
        &app,
        &team_uri(teams[0]),
        &ana,
        json!({ "captain": p[1].id }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{f}");
    assert_eq!(team_of(&f, &p[0]).unwrap()["captain"], p[1].id);
    assert_eq!(
        patch(
            &app,
            &team_uri(teams[0]),
            &ana,
            json!({ "captain": p[2].id })
        )
        .await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("team.captain_invalid")
        )
    );
    // `null` tira o nome.
    let (_, f) = patch(&app, &team_uri(teams[0]), &ana, json!({ "name": null })).await;
    assert!(team_of(&f, &p[0]).unwrap()["name"].is_null());

    // Encerrado, nada muda.
    set_status(&db, "copa", "finished").await;
    assert_eq!(
        patch(&app, &team_uri(teams[0]), &ana, json!({ "name": "Y" })).await,
        (StatusCode::CONFLICT, err("tournament.finished"))
    );
}

#[tokio::test]
async fn the_organizer_substitutes_a_player_during_the_tournament() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 5).await;
    let who = [&p[0], &p[1], &p[2], &p[3]];
    let teams = formed(&app, &ana, tdm(2, single(3, 3)), 2, &who).await;
    register(&app, "copa", &[&p[4]]).await;
    let sub = |out: &User, sub: &User| json!({ "out": out.id, "in": sub.id });
    let uri = format!("{}/substitution", team_uri(teams[0]));
    // Antes de começar, a formação resolve.
    assert_eq!(
        post(&app, &uri, &ana, sub(&p[0], &p[4])).await,
        (StatusCode::CONFLICT, err("tournament.not_in_progress"))
    );
    let (status, seeded) = put(
        &app,
        &format!("{COPA}/seeding"),
        &ana,
        json!({ "team_ids": teams }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{seeded}");
    let (status, body) = post_empty(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // O mapa 1 com P1 e P2.
    let (status, body) = report(&app, &p[1], "wb-1-1", 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // P1 (o capitão) sai e o reserva P5 entra: a capitania passa para P2.
    let (status, f) = post(&app, &uri, &ana, sub(&p[0], &p[4])).await;
    assert_eq!(status, StatusCode::OK, "{f}");
    let team = team_of(&f, &p[4]).unwrap();
    assert_eq!(team["id"], teams[0]);
    assert_eq!(team["captain"], p[1].id);
    assert!(team_of(&f, &p[0]).is_none());
    assert_eq!(reserves(&f), [p[0].id]);

    // A Bia, que não estava inscrita, entra no lugar de P2.
    let (status, f) = post(&app, &uri, &ana, sub(&p[1], &bia)).await;
    assert_eq!(status, StatusCode::OK, "{f}");
    assert_eq!(team_of(&f, &bia).unwrap()["id"], teams[0]);

    // O mapa 1 continua com quem jogou; o 2 já é com os novos.
    let (status, _) = confirm(&app, &p[2], "wb-1-1", 1).await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = report(&app, &p[4], "wb-1-1", 2, game("a", "exile")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let d = detail(&app, &ana, "wb-1-1").await;
    let lineup = |n: usize, side: &str| -> Vec<i64> {
        let mut ids: Vec<i64> = d["games"][n][format!("lineup_{side}")]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| m["user_id"].as_i64().unwrap())
            .collect();
        ids.sort();
        ids
    };
    let sorted = |mut ids: Vec<i64>| {
        ids.sort();
        ids
    };
    assert_eq!(lineup(0, "a"), sorted(vec![p[0].id, p[1].id]));
    assert_eq!(lineup(1, "a"), sorted(vec![p[4].id, bia.id]));

    // Quem entra não pode estar em outro time; quem sai é do time.
    assert_eq!(
        post(&app, &uri, &ana, sub(&p[4], &p[2])).await,
        (StatusCode::CONFLICT, err("team.player_in_team"))
    );
    assert_eq!(
        post(&app, &uri, &ana, sub(&p[2], &p[0])).await,
        (StatusCode::UNPROCESSABLE_ENTITY, err("team.member_invalid"))
    );
    assert_eq!(
        post(&app, &uri, &bia, sub(&p[4], &p[0])).await.0,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn whoever_leaves_the_community_leaves_the_team_short() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    let who = [&p[0], &p[1], &p[2], &p[3]];
    started(&app, &ana, tdm(2, json!({})), 2, &who).await;

    // P1, o capitão, sai: o time joga com P2, que vira o capitão.
    let (status, body) = delete(&app, "/tenants/quake-br/me", &p[0]).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    let d = detail(&app, &p[1], "wb-1-1").await;
    assert!(d["summary"]["result"].is_null(), "sem W.O.: {d}");
    assert_eq!(d["a"]["captain"], p[1].id);
    assert_eq!(d["a"]["withdrawn"], false);

    // O mapa enviado agora tem só P2 do lado a.
    let (status, body) = report(&app, &p[1], "wb-1-1", 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let lineup = &body["games"][0]["lineup_a"];
    assert_eq!(lineup.as_array().unwrap().len(), 1, "{lineup}");
    assert_eq!(lineup[0]["user_id"], p[1].id);

    // Sem ninguém na comunidade, o time desiste e toma W.O.
    let (status, _) = delete(&app, "/tenants/quake-br/me", &p[1]).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let d = detail(&app, &ana, "wb-1-1").await;
    assert_eq!(d["summary"]["result"]["walkover"], true, "{d}");
    assert_eq!(d["summary"]["result"]["winner"], "b");
}
