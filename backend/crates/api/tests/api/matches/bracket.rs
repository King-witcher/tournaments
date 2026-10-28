//! A chave nasce no início, segue a visibilidade do torneio e some ao
//! desfazer o início.

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;

use super::*;
use crate::common::*;

#[sqlx::test(migrations = "../../migrations")]
async fn start_generates_the_bracket_in_seed_order(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    start(&app, &ana, copa(json!({})), &[&p[0], &p[1], &p[2], &p[3]]).await;

    let b = bracket(&app, &p[0]).await;
    assert_eq!(b["status"], "in_progress");
    assert_eq!(b["format"]["bracket"], "double");
    assert_eq!(b["contest_minutes"], 5);
    assert_eq!(b["results"], 0);
    let seeds: Vec<(i64, i64)> = b["entrants"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| (e["user_id"].as_i64().unwrap(), e["seed"].as_i64().unwrap()))
        .collect();
    assert_eq!(
        seeds,
        [(p[0].id, 1), (p[1].id, 2), (p[2].id, 3), (p[3].id, 4)]
    );
    let keys: Vec<&str> = b["matches"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["key"].as_str().unwrap())
        .collect();
    assert_eq!(
        keys,
        [
            "wb-1-1", "wb-1-2", "wb-2-1", "lb-1-1", "lb-2-1", "gf-1-1", "gf-2-1"
        ]
    );

    // 1 x 4 e 2 x 3 na primeira rodada; o resto espera.
    let first = find(&b, "wb-1-1");
    assert_eq!(sides(first), (json!(p[0].id), json!(p[3].id)));
    assert_eq!(
        (&first["state"], &first["best_of"], &first["result"]),
        (&json!("ready"), &json!(3), &Value::Null)
    );
    assert_eq!(
        (&first["yours"], &first["task"]),
        (&json!(true), &json!("report"))
    );
    assert_eq!(sides(find(&b, "wb-1-2")), (json!(p[1].id), json!(p[2].id)));
    assert_eq!(find(&b, "wb-1-2")["task"], Value::Null);
    let final_ = find(&b, "wb-2-1");
    assert_eq!(final_["state"], "pending");
    assert_eq!(
        final_["a"]["from"],
        json!({ "outcome": "winner", "match_key": "wb-1-1" })
    );
    assert_eq!(find(&b, "lb-1-1")["b"]["from"]["outcome"], "loser");
    assert_eq!(find(&b, "gf-1-1")["best_of"], 5);
    assert_eq!(b["standings"], json!([]));

    // O organizador vê sem tarefa nas partidas prontas.
    let seen = bracket(&app, &ana).await;
    assert_eq!(find(&seen, "wb-1-1")["yours"], false);
    assert_eq!(find(&seen, "wb-1-1")["task"], Value::Null);

    let (status, m) = get(&app, &format!("{COPA}/matches/wb-1-1"), &p[3]).await;
    assert_eq!(status, StatusCode::OK, "{m}");
    assert_eq!(m["a"]["nickname"], "P1");
    assert_eq!(m["b"]["seed"], 4);
    assert_eq!(
        (&m["games"], &m["history"], &m["can_manage"]),
        (&json!([]), &json!([]), &json!(false))
    );
    for bad in ["wb-9-9", "wb-01-1", "xx"] {
        assert_eq!(
            get(&app, &format!("{COPA}/matches/{bad}"), &p[0]).await,
            (StatusCode::NOT_FOUND, err("match.not_found")),
            "{bad}"
        );
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn byes_advance_without_playing(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 3).await;
    start(&app, &ana, copa(single(3, 5)), &[&p[0], &p[1], &p[2]]).await;

    let b = bracket(&app, &p[0]).await;
    let bye = find(&b, "wb-1-1");
    assert_eq!(bye["state"], "bye");
    assert_eq!(
        (&bye["b"]["bye"], &bye["task"]),
        (&json!(true), &Value::Null)
    );
    // O seed 1 já está na final, esperando o vencedor de 2 x 3.
    let final_ = find(&b, "wb-2-1");
    assert_eq!(
        (
            &final_["state"],
            &final_["a"]["user_id"],
            &final_["best_of"]
        ),
        (&json!("pending"), &json!(p[0].id), &json!(5))
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn start_rechecks_the_champion_pool_with_the_real_entrants(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 3).await;
    // md1 em todas, sem repetir no torneio, 1 ban: com 3 participantes, o
    // pior caso são 2 mapas, então precisa de 3 campeões.
    let body = copa(with(
        single(1, 1),
        json!({
            "max_participants": 4,
            "champion_rules": {
                "bans_per_game": 1,
                "no_repeat": { "who": "player", "scope": "tournament" }
            }
        }),
    ));
    create_tournament(&app, &ana, body).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&p[0], &p[1], &p[2]]).await;
    sqlx::query!("update champions set active = false where slug not in ('anarki', 'athena')")
        .execute(&db)
        .await
        .unwrap();

    assert_eq!(
        post_empty(&app, &format!("{COPA}/start"), &ana).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({ "error": "champions.pool_exhausted", "params": { "required": 3, "available": 2 } })
        )
    );
    // Nada mudou: nem status nem seeds.
    let (_, seen) = get(&app, COPA, &ana).await;
    assert_eq!(seen["status"], "registration");
    let seeded =
        sqlx::query_scalar!(r#"select count(*) as "n!" from participants where seed is not null"#)
            .fetch_one(&db)
            .await
            .unwrap();
    assert_eq!(seeded, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn bracket_follows_the_tournament_visibility(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    create_tournament(&app, &ana, copa(json!({ "visibility": "private" }))).await;
    publish(&app, &ana, "copa").await;
    let add = format!("{COPA}/participants");
    for who in &p {
        let (status, _) = post(&app, &add, &ana, json!({ "user_id": who.id })).await;
        assert_eq!(status, StatusCode::CREATED);
    }
    assert_eq!(
        get(&app, &format!("{COPA}/bracket"), &ana).await,
        (StatusCode::NOT_FOUND, err("bracket.not_found"))
    );
    let (status, _) = post_empty(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK);

    // Bia é da comunidade mas não está no privado.
    let hidden = (StatusCode::NOT_FOUND, err("tournament.not_found"));
    assert_eq!(get(&app, &format!("{COPA}/bracket"), &bia).await, hidden);
    assert_eq!(
        get(&app, &format!("{COPA}/matches/wb-1-1"), &bia).await,
        hidden
    );
    let outsider = user(&db, "Zé").await;
    assert_eq!(
        get(&app, &format!("{COPA}/bracket"), &outsider).await,
        (StatusCode::NOT_FOUND, err("tenant.not_found"))
    );
    assert_eq!(bracket(&app, &p[1]).await["status"], "in_progress");
    assert_eq!(
        bracket(&app, &ana).await["entrants"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn undo_start_drops_the_bracket(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(json!({})), &[&p[0], &p[1]]).await;
    let (status, undone) = delete(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{undone}");
    assert_eq!(undone["status"], "registration");
    assert_eq!(
        get(&app, &format!("{COPA}/bracket"), &p[0]).await,
        (StatusCode::NOT_FOUND, err("bracket.not_found"))
    );
}
