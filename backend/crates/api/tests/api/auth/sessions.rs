//! Sessões da conta: listar, encerrar uma e encerrar as outras.

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use chrono::{DateTime, TimeDelta};
use serde_json::{Value, json};
use sqlx::PgPool;

use super::{body_text, pair, send, set_cookie, signup};
use crate::common::{User, app, call, delete, err, get, user};

const PASSWORD: &str = "senha forte 1";

/// Entra como `username` pelo navegador `agent` e devolve a sessão nova.
async fn login_from(app: &Router, username: &str, agent: &str) -> User {
    let req = Request::post("/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::USER_AGENT, agent)
        .body(Body::from(
            json!({ "username": username, "password": PASSWORD }).to_string(),
        ))
        .unwrap();
    let res = send(app, req).await;
    assert_eq!(res.status(), StatusCode::OK);
    let cookie = pair(&set_cookie(&res, "session").unwrap());
    let me: Value = serde_json::from_str(&body_text(res).await).unwrap();
    User {
        id: me["id"].as_i64().unwrap(),
        cookie,
    }
}

async fn list(app: &Router, who: &User) -> Vec<Value> {
    let (status, body) = get(app, "/auth/sessions", who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body.as_array().unwrap().clone()
}

async fn me_status(app: &Router, who: &User) -> StatusCode {
    get(app, "/me", who).await.0
}

#[sqlx::test(migrations = "../../migrations")]
async fn sessions_are_listed_newest_first_with_the_browser(db: PgPool) {
    let app = app(db.clone());
    signup(&app, "Rapha", PASSWORD).await;
    let pc = login_from(&app, "rapha", "Firefox/140").await;
    login_from(&app, "rapha", "Celular").await;

    let sessions = list(&app, &pc).await;
    assert_eq!(sessions.len(), 3);
    assert_eq!(sessions[0]["user_agent"], "Celular");
    assert_eq!(sessions[0]["current"], false);
    assert_eq!(sessions[1]["user_agent"], "Firefox/140");
    assert_eq!(sessions[1]["current"], true);
    // O cadastro dos testes não manda User-Agent.
    assert!(sessions[2]["user_agent"].is_null());

    let id = sessions[0]["id"].as_str().unwrap();
    assert_eq!(id.len(), 16);
    assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    // Dura 30 dias (a expiração é em segundos; a criação, em milissegundos).
    let at =
        |field: &str| DateTime::parse_from_rfc3339(sessions[0][field].as_str().unwrap()).unwrap();
    let lasts = at("expires_at") - at("created_at");
    assert!(lasts <= TimeDelta::days(30) && lasts > TimeDelta::days(30) - TimeDelta::seconds(1));

    // Cada conta vê só as próprias.
    let other = user(&db, "Outro").await;
    assert_eq!(list(&app, &other).await.len(), 1);

    let (status, body) = call(&app, Method::GET, "/auth/sessions", None, None).await;
    assert_eq!(
        (status, body),
        (StatusCode::UNAUTHORIZED, err("auth.unauthenticated"))
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn sessions_end_one_by_one_or_all_others(db: PgPool) {
    let app = app(db.clone());
    let first = signup(&app, "Rapha", PASSWORD).await;
    let pc = login_from(&app, "rapha", "Firefox/140").await;
    let phone = login_from(&app, "rapha", "Celular").await;
    let other = user(&db, "Outro").await;

    // Encerra a do celular pelo id da lista.
    let id = list(&app, &pc).await[0]["id"].as_str().unwrap().to_owned();
    let (status, _) = delete(&app, &format!("/auth/sessions/{id}"), &pc).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(me_status(&app, &phone).await, StatusCode::UNAUTHORIZED);
    assert_eq!(me_status(&app, &pc).await, StatusCode::OK);
    assert_eq!(list(&app, &pc).await.len(), 2);

    // Id que não é de uma sessão da conta: nem outro prefixo, nem a de outra pessoa.
    let theirs = list(&app, &other).await[0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    for missing in [id.as_str(), &id[..8], "0000000000000000", theirs.as_str()] {
        let (status, body) = delete(&app, &format!("/auth/sessions/{missing}"), &pc).await;
        assert_eq!(
            (status, body),
            (StatusCode::NOT_FOUND, err("auth.session_not_found")),
            "{missing}"
        );
    }
    assert_eq!(me_status(&app, &other).await, StatusCode::OK);

    // As outras saem; esta fica.
    let (status, _) = delete(&app, "/auth/sessions", &pc).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(me_status(&app, &first).await, StatusCode::UNAUTHORIZED);
    let sessions = list(&app, &pc).await;
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0]["current"], true);

    // Encerrar a própria é sair: o cookie vai junto.
    let own = sessions[0]["id"].as_str().unwrap();
    let req = Request::delete(format!("/auth/sessions/{own}"))
        .header(header::COOKIE, &pc.cookie)
        .body(Body::empty())
        .unwrap();
    let res = send(&app, req).await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    assert!(set_cookie(&res, "session").unwrap().contains("Max-Age=0"));
    assert_eq!(me_status(&app, &pc).await, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn session_of_a_deleted_account_lets_no_one_in(db: PgPool) {
    let app = app(db.clone());
    let gone = user(&db, "Some").await;
    assert_eq!(me_status(&app, &gone).await, StatusCode::OK);

    sqlx::query!("delete from users where id = $1", gone.id)
        .execute(&db)
        .await
        .unwrap();
    assert_eq!(me_status(&app, &gone).await, StatusCode::UNAUTHORIZED);
}
