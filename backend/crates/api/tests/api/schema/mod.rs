//! Regras do banco para o schema inteiro, gravando o estado inválido direto
//! por SQL: o que cada exclusão apaga ou recusa (`deletion`), o owner de cada
//! tenant (`owner`), o membro inativo (`inactive`) e a coerência e a forma
//! dos dados (`checks`). As de resultados e histórico ficam em
//! `matches::schema`.

mod checks;
mod deletion;
mod inactive;
mod owner;

use axum::{Router, http::StatusCode};
use serde_json::json;
use sqlx::PgPool;

use crate::common::*;
use crate::matches::{COPA, copa, report, set_result, single};

pub const CHECK: &str = "23514";
pub const FOREIGN_KEY: &str = "23503";
pub const UNIQUE: &str = "23505";

/// Código do Postgres e nome da regra do erro (ex.: `("23503",
/// "users_owner_delete")`).
pub fn rejected<T: std::fmt::Debug>(result: Result<T, sqlx::Error>) -> (String, String) {
    let err = result.expect_err("o banco devia recusar");
    let db = err
        .as_database_error()
        .unwrap_or_else(|| panic!("não é erro do banco: {err}"));
    (
        db.code().map(|c| c.into_owned()).unwrap_or_default(),
        db.constraint().unwrap_or_default().to_owned(),
    )
}

/// `rejected` esperado: código e nome da regra.
pub fn rule(code: &str, name: &str) -> (String, String) {
    (code.to_owned(), name.to_owned())
}

pub async fn exec(db: &PgPool, sql: &str) -> Result<u64, sqlx::Error> {
    let sql = sqlx::AssertSqlSafe(sql.to_owned());
    Ok(sqlx::query(sql).execute(db).await?.rows_affected())
}

/// Resultado de um `select count(*)`.
pub async fn count(db: &PgPool, sql: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(sql.to_owned()))
        .fetch_one(db)
        .await
        .unwrap()
}

/// `quake-br` com um torneio encerrado e tudo o que pode haver dentro dele.
pub struct Cup {
    pub app: Router,
    /// Owner.
    pub ana: User,
    /// Player sem inscrição.
    pub bia: User,
    /// Organizer: criou um convite do tenant e o link de inscrição, inscreveu
    /// `p1`, marcou o check-in dele e resolveu a final.
    pub olga: User,
    /// Entrou pelo convite da Olga.
    pub ze: User,
    /// Os finalistas: `p1` (seed 1, inscrito pela Olga) venceu `p2` (seed 2,
    /// inscrito pelo link) por 2 x 1, com detalhe por mapa.
    pub p1: User,
    pub p2: User,
    pub tenant: i64,
    pub tournament: i64,
}

/// Monta o `Cup`: final única em md3 contestada e resolvida pela organizadora.
pub async fn finished_cup(db: &PgPool) -> Cup {
    let (app, ana, bia) = quake_br(db).await;
    let olga = member(&app, db, &ana, "Olga").await;
    let uri = format!("/tenants/quake-br/members/{}/role", olga.id);
    let (status, body) = put(&app, &uri, &ana, json!({ "role": "organizer" })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let ze = user(db, "Zé").await;
    join(&app, &ze, &invite(&app, &olga, "quake-br").await).await;
    let p1 = member(&app, db, &ana, "P1").await;

    create_tournament(&app, &ana, copa(single(3, 3))).await;
    publish(&app, &ana, "copa").await;
    let (status, body) = put(
        &app,
        &format!("{COPA}/translations/es"),
        &ana,
        json!({ "name": "Copa de Octubre" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = post(
        &app,
        &format!("{COPA}/participants"),
        &olga,
        json!({ "user_id": p1.id }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let (status, link) = post_empty(&app, &format!("{COPA}/invites"), &olga).await;
    assert_eq!(status, StatusCode::CREATED, "{link}");
    let p2 = user(db, "P2").await;
    join(&app, &p2, link["token"].as_str().unwrap()).await;
    // Check-in marcado pela organizadora (este torneio não tem janela, mas a
    // coluna vale igual).
    sqlx::query(
        "update participants set checked_in_at = now(), checked_in_by = $1 where user_id = $2",
    )
    .bind(olga.id)
    .bind(p1.id)
    .execute(db)
    .await
    .unwrap();

    let (status, body) = put(
        &app,
        &format!("{COPA}/seeding"),
        &ana,
        json!({ "user_ids": [p1.id, p2.id] }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = post_empty(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let game = |map: &str, a: u16, b: u16| json!({ "map": map, "a_frags": a, "b_frags": b });
    let games = json!([
        game("awoken", 20, 10),
        game("exile", 5, 10),
        game("insomnia", 30, 2)
    ]);
    let (status, body) = report(
        &app,
        &p1,
        "wb-1-1",
        json!({ "a_wins": 2, "b_wins": 1, "games": games }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let uri = format!("{COPA}/matches/wb-1-1/contest");
    let (status, body) = post(&app, &uri, &p2, json!({ "comment": "foi 2 x 0" })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let result = json!({ "kind": "played", "a_wins": 2, "b_wins": 1, "games": games });
    let (status, body) = set_result(&app, &olga, "wb-1-1", result, Some("conferido")).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let t = sqlx::query!("select tenant_id, id, status from tournaments where slug = 'copa'")
        .fetch_one(db)
        .await
        .unwrap();
    assert_eq!(t.status, "finished");
    assert_eq!(count(db, "select count(*) from match_games").await, 3);
    assert_eq!(count(db, "select count(*) from match_events").await, 3);
    Cup {
        app,
        ana,
        bia,
        olga,
        ze,
        p1,
        p2,
        tenant: t.tenant_id,
        tournament: t.id,
    }
}
