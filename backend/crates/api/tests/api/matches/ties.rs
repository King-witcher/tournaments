//! O empate do fim da fase decidido pela organização: a fase espera, o aviso
//! sai uma vez, a decisão por grupo (com o papel e o grupo que bate), um grupo
//! de cada vez até o último, a correção que muda o grupo, e o padrão, que não
//! espera.

use axum::{Router, http::StatusCode};
use serde_json::{Value, json};

use super::*;
use crate::common::*;
use crate::realtime::{clear, notices, outbox};

const TIES: &str = "/tenants/quake-br/tournaments/copa/ties";

/// Todos contra todos em md1 sem desempates, com `final_tie`.
fn cup(final_tie: Option<&str>) -> Value {
    let mut fields = json!({ "best_of": 1, "tiebreakers": [] });
    if let Some(f) = final_tie {
        fields["final_tie"] = json!(f);
    }
    copa(table("round_robin", fields))
}

/// O organizador dá W.O. a `winner` na série entre `winner` e `loser`.
async fn beat(app: &Router, ana: &User, winner: &User, loser: &User) {
    let b = bracket(app, ana).await;
    let m = b["matches"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| {
            let (a, x) = sides(&b, m);
            (a == winner.id && x == loser.id) || (a == loser.id && x == winner.id)
        })
        .unwrap();
    let side = if sides(&b, m).0 == winner.id {
        "a"
    } else {
        "b"
    };
    let key = m["key"].as_str().unwrap();
    let (status, body) = set_result(app, ana, key, walkover(side), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

/// Os jogadores da tabela, em ordem de posição.
fn order(b: &Value) -> Vec<Value> {
    b["table"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| player(b, &r["team_id"]))
        .collect()
}

fn ids(users: &[&User]) -> Vec<Value> {
    users.iter().map(|u| json!(u.id)).collect()
}

/// O time de `who` na chave `b`.
fn team(b: &Value, who: &User) -> Value {
    table_row(b, who)["team_id"].clone()
}

async fn tie_notices(db: &sqlx::PgPool) -> Vec<Vec<i64>> {
    notices(&outbox(db).await)
        .into_iter()
        .filter(|(code, ..)| code == "tournament.tie_pending")
        .map(|(_, to, _)| to)
        .collect()
}

async fn organizing(app: &Router, who: &User) -> Value {
    let (status, body) = get(app, "/me/organizing", who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["pending_ties"].clone()
}

#[tokio::test]
async fn the_last_tie_waits_for_the_organizer() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 3).await;
    start(&app, &ana, cup(Some("organizer")), &[&p[0], &p[1], &p[2]]).await;
    beat(&app, &ana, &p[0], &p[1]).await;
    beat(&app, &ana, &p[1], &p[2]).await;
    // Durante a fase, nada pendente.
    let b = bracket(&app, &ana).await;
    assert_eq!(b["pending_ties"], json!([]));
    assert!(tie_notices(&db).await.is_empty());
    beat(&app, &ana, &p[2], &p[0]).await;

    // Todas decididas, os três com saldo 0: a fase espera.
    let b = bracket(&app, &p[1]).await;
    assert_eq!(b["status"], "in_progress");
    let teams: Vec<Value> = p.iter().map(|u| team(&b, u)).collect();
    assert_eq!(
        b["pending_ties"],
        json!([{ "position": 1, "team_ids": teams }])
    );
    for row in b["table"].as_array().unwrap() {
        assert_eq!(
            (&row["position"], &row["tie_pending"]),
            (&json!(1), &json!(true))
        );
    }
    assert_eq!(b["standings"], json!([]));
    assert_eq!(tie_notices(&db).await, [vec![ana.id]]);
    assert_eq!(
        organizing(&app, &ana).await[0]["tournament"]["slug"],
        "copa"
    );
    let (status, mine) = get(&app, "/me/tournaments", &p[1]).await;
    assert_eq!(status, StatusCode::OK, "{mine}");
    let entry = &mine["registered"][0];
    assert_eq!(entry["situation"], "tie_pending");
    assert_eq!(entry["placement"], json!({ "rank": 1, "tied": 3 }));

    // Só a organização decide, e só um grupo que está pendente.
    let decision =
        |teams: &[Value], reason: Option<&str>| json!({ "teams": teams, "reason": reason });
    let full = [teams[2].clone(), teams[0].clone(), teams[1].clone()];
    let (status, body) = post(&app, TIES, &p[0], decision(&full, None)).await;
    assert_eq!(
        (status, body),
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
    let (status, body) = post(&app, TIES, &ana, decision(&full[..2], None)).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("tie.not_pending"))
    );
    let (status, body) = post(&app, TIES, &ana, decision(&full, Some(" "))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"], "tie.reason_invalid");

    // A decisão ordena o grupo e encerra o torneio, sem outro aviso.
    clear(&db).await;
    let (status, b) = post(
        &app,
        TIES,
        &ana,
        decision(&full, Some("Melhor de 3 na live")),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{b}");
    assert_eq!(b["status"], "finished");
    assert_eq!(b["pending_ties"], json!([]));
    assert_eq!(order(&b), ids(&[&p[2], &p[0], &p[1]]));
    let rows = b["table"].as_array().unwrap();
    let ahead: Vec<&Value> = rows.iter().map(|r| &r["ahead_by"]).collect();
    assert_eq!(
        ahead,
        [&json!("organizer"), &json!("organizer"), &Value::Null]
    );
    assert!(
        rows.iter()
            .all(|r| r["tie_reason"] == "Melhor de 3 na live")
    );
    assert!(rows.iter().all(|r| r["tie_pending"] == false));
    assert_eq!(b["standings"].as_array().unwrap().len(), 3);
    assert!(tie_notices(&db).await.is_empty());
    assert_eq!(organizing(&app, &ana).await, json!([]));
    let (status, body) = post(&app, TIES, &ana, decision(&full, None)).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("tie.not_pending"))
    );
}

#[tokio::test]
async fn each_group_is_decided_on_its_own_and_a_correction_changes_the_group() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    let all = [&p[0], &p[1], &p[2], &p[3]];
    start(&app, &ana, cup(Some("organizer")), &all).await;
    // P1 vence todos; P2 > P3 > P4 > P2.
    for (w, l) in [(0, 1), (0, 2), (0, 3), (1, 2), (2, 3), (3, 1)] {
        beat(&app, &ana, &p[w], &p[l]).await;
    }
    let b = bracket(&app, &ana).await;
    let teams: Vec<Value> = p.iter().map(|u| team(&b, u)).collect();
    assert_eq!(
        b["pending_ties"],
        json!([{ "position": 2, "team_ids": teams[1..] }])
    );
    // O 1º já tem a posição.
    assert_eq!(
        standings(&b),
        json!([{ "user_id": p[0].id, "rank": 1, "tied": 1 }])
    );
    let (status, b) = post(
        &app,
        TIES,
        &ana,
        json!({ "teams": [&teams[3], &teams[2], &teams[1]] }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{b}");
    assert_eq!(b["status"], "finished");
    assert_eq!(order(&b), ids(&[&p[0], &p[3], &p[2], &p[1]]));
    assert_eq!(table_row(&b, &p[3])["tie_reason"], Value::Null);

    // A correção muda os empatados: a decisão deixa de valer, o torneio
    // reabre esperando os dois grupos novos, e a organização é avisada de novo.
    clear(&db).await;
    beat(&app, &ana, &p[1], &p[0]).await;
    let b = bracket(&app, &ana).await;
    assert_eq!(b["status"], "in_progress");
    assert_eq!(
        b["pending_ties"],
        json!([
            { "position": 1, "team_ids": [&teams[0], &teams[1]] },
            { "position": 3, "team_ids": [&teams[2], &teams[3]] },
        ])
    );
    assert_eq!(tie_notices(&db).await, [vec![ana.id]]);

    // Na ordem da chave, um grupo por vez e em qualquer ordem: o de baixo não
    // encerra, sobra o outro; o último encerra.
    let (status, b) = post(&app, TIES, &ana, json!({ "teams": teams[2..] })).await;
    assert_eq!(status, StatusCode::OK, "{b}");
    assert_eq!(b["status"], "in_progress");
    assert_eq!(
        b["pending_ties"],
        json!([{ "position": 1, "team_ids": teams[..2] }])
    );
    // O mesmo grupo de novo, como quem ainda não viu que a outra pessoa decidiu.
    let (status, body) = post(&app, TIES, &ana, json!({ "teams": teams[2..] })).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("tie.not_pending"))
    );
    let (status, b) = post(&app, TIES, &ana, json!({ "teams": teams[..2] })).await;
    assert_eq!(status, StatusCode::OK, "{b}");
    assert_eq!(b["status"], "finished");
    assert_eq!(order(&b), ids(&all));
}

#[tokio::test]
async fn by_default_the_seed_decides_and_nothing_waits() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 3).await;
    start(&app, &ana, cup(None), &[&p[0], &p[1], &p[2]]).await;
    let (status, t) = get(&app, "/tenants/quake-br/tournaments/copa", &ana).await;
    assert_eq!(status, StatusCode::OK, "{t}");
    assert_eq!(t["stages"][0]["final_tie"], "seed");
    for (w, l) in [(0, 1), (1, 2), (2, 0)] {
        beat(&app, &ana, &p[w], &p[l]).await;
    }
    let b = bracket(&app, &ana).await;
    assert_eq!(b["status"], "finished");
    assert_eq!(b["pending_ties"], json!([]));
    assert_eq!(order(&b), ids(&[&p[0], &p[1], &p[2]]));
    assert!(tie_notices(&db).await.is_empty());
    let teams: Vec<Value> = p.iter().map(|u| team(&b, u)).collect();
    let (status, body) = post(&app, TIES, &ana, json!({ "teams": teams })).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("tie.not_pending"))
    );
}
