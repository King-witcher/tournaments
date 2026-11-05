//! Sessões da conta: listar, encerrar uma e encerrar as outras; a memória que
//! responde às requisições e o Redis que a guarda entre os boots.

use std::sync::Arc;

use api::{
    AppState,
    auth::{AuthSettings, SessionStore},
    shared::{
        Redis,
        token::{hash_token, hex},
    },
};
use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use chrono::{DateTime, TimeDelta, Utc};
use redis::Cmd;
use serde_json::{Value, json};
use sqlx::PgPool;

use super::{body_text, pair, send, set_cookie, signup};
use crate::common::{
    User, WEB, app, app_from, call, delete, err, get, redis, sessions, state, user,
};

const PASSWORD: &str = "senha forte 1";

/// Entra com `email` pelo navegador `agent` e devolve a sessão nova.
async fn login_from(app: &Router, email: &str, agent: &str) -> User {
    let req = Request::post("/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::USER_AGENT, agent)
        .body(Body::from(
            json!({ "email": email, "password": PASSWORD }).to_string(),
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

#[tokio::test]
async fn sessions_are_listed_newest_first_with_the_browser() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    signup(&app, &db, "Rapha", PASSWORD).await;
    let pc = login_from(&app, "rapha@example.com", "Firefox/140").await;
    login_from(&app, "rapha@example.com", "Celular").await;

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
    let at =
        |field: &str| DateTime::parse_from_rfc3339(sessions[0][field].as_str().unwrap()).unwrap();
    assert_eq!(at("expires_at") - at("created_at"), TimeDelta::days(30));

    // Cada conta vê só as próprias.
    let other = user(&db, "Outro").await;
    assert_eq!(list(&app, &other).await.len(), 1);

    let (status, body) = call(&app, Method::GET, "/auth/sessions", None, None).await;
    assert_eq!(
        (status, body),
        (StatusCode::UNAUTHORIZED, err("auth.unauthenticated"))
    );
}

#[tokio::test]
async fn sessions_end_one_by_one_or_all_others() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let first = signup(&app, &db, "Rapha", PASSWORD).await;
    let pc = login_from(&app, "rapha@example.com", "Firefox/140").await;
    let phone = login_from(&app, "rapha@example.com", "Celular").await;
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

/// O front chama esta rota para tirar o cookie que a API recusou: ele sai com o
/// domínio e o caminho que o navegador guardou.
#[tokio::test]
async fn clearing_removes_the_cookie_of_a_dead_session() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app_from(AppState {
        auth: Arc::new(AuthSettings {
            discord: None,
            google: None,
            web_origin: WEB.into(),
            cookie_name: "session_staging".into(),
            cookie_domain: Some(".staging.telefrag.com.br".into()),
            cookie_secure: true,
        }),
        ..state(db)
    });
    let req = Request::post("/auth/sessions/clear-expired")
        .header(header::COOKIE, "session=da-producao; session_staging=morta")
        .body(Body::empty())
        .unwrap();
    let res = send(&app, req).await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    let removal = set_cookie(&res, "session_staging").unwrap();
    for part in ["Max-Age=0", "Domain=staging.telefrag.com.br", "Path=/"] {
        assert!(removal.contains(part), "{removal}");
    }
    assert_eq!(set_cookie(&res, "session"), None, "o da produção fica");
}

/// Entre o servidor ver o cookie morto e o navegador pedir a limpeza, outra aba
/// pode ter entrado: a sessão nova fica, e o cookie dela também.
#[tokio::test]
async fn clearing_keeps_a_live_session() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = user(&db, "Some").await;
    let req = Request::post("/auth/sessions/clear-expired")
        .header(header::COOKIE, &me.cookie)
        .body(Body::empty())
        .unwrap();
    let res = send(&app, req).await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    assert_eq!(set_cookie(&res, "session"), None);
    assert_eq!(me_status(&app, &me).await, StatusCode::OK);
}

#[tokio::test]
async fn session_of_a_deleted_account_lets_no_one_in() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let gone = user(&db, "Some").await;
    assert_eq!(me_status(&app, &gone).await, StatusCode::OK);

    sqlx::query!("delete from users where id = $1", gone.id)
        .execute(&db)
        .await
        .unwrap();
    assert_eq!(me_status(&app, &gone).await, StatusCode::UNAUTHORIZED);
}

/// O app de um processo novo: a memória dele sobe do Redis.
async fn booted(db: &PgPool, read_through: bool) -> Router {
    let sessions = SessionStore::load(redis(db), read_through).await.unwrap();
    app_from(AppState {
        sessions,
        ..state(db.clone())
    })
}

/// Grava uma sessão como as de antes (hash do Redis), que ainda vivem por até 30
/// dias, e como os scripts locais criam, e devolve o cookie dela.
async fn old_format_session(redis: &Redis, user_id: i64, token: &str) -> User {
    let key = redis.key(format_args!("session:{}", hex(&hash_token(token))));
    let created = Utc::now().timestamp_millis();
    let fields = [
        ("user_id", user_id.to_string()),
        ("created_at", created.to_string()),
        ("user_agent", "Firefox/140".to_owned()),
    ];
    redis
        .query::<()>(&Cmd::hset_multiple(&key, &fields))
        .await
        .unwrap();
    User {
        id: user_id,
        cookie: format!("session={token}"),
    }
}

/// O que importa do Redis: o boot traz quem estava logado, e não traz quem saiu.
#[tokio::test]
async fn a_boot_brings_back_what_redis_kept() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let stays = user(&db, "Fica").await;
    let leaves = user(&db, "Sai").await;
    let (status, _) = call(&app, Method::POST, "/auth/logout", Some(&leaves), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let old = old_format_session(&redis(&db), stays.id, "sessao-de-antes").await;

    let rebooted = booted(&db, false).await;
    assert_eq!(me_status(&rebooted, &stays).await, StatusCode::OK);
    assert_eq!(
        me_status(&rebooted, &leaves).await,
        StatusCode::UNAUTHORIZED
    );

    assert_eq!(me_status(&rebooted, &old).await, StatusCode::OK);
    let listed = list(&rebooted, &old).await;
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0]["user_agent"], "Firefox/140");
    let (status, _) = delete(&rebooted, "/auth/sessions", &stays).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let again = booted(&db, false).await;
    assert_eq!(me_status(&again, &old).await, StatusCode::UNAUTHORIZED);
}

/// A instância antiga atende até a nova ficar no ar: o que ela grava no Redis só
/// chega à nova na segunda leitura, e até lá o Redis não é consultado.
#[tokio::test]
async fn reloading_brings_what_the_old_instance_wrote() {
    let (db, _lease) = crate::db::test_db().await;
    let ended = user(&db, "Encerrada").await;
    let new = SessionStore::load(redis(&db), false).await.unwrap();
    let new_app = app_from(AppState {
        sessions: new.clone(),
        ..state(db.clone())
    });
    assert_eq!(me_status(&new_app, &ended).await, StatusCode::OK);

    let old_app = app(db.clone());
    let (status, _) = call(&old_app, Method::POST, "/auth/logout", Some(&ended), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let token = sessions(&db).create(ended.id, None).await.unwrap();
    let created = User {
        id: ended.id,
        cookie: format!("session={token}"),
    };
    assert_eq!(me_status(&new_app, &ended).await, StatusCode::OK);
    assert_eq!(
        me_status(&new_app, &created).await,
        StatusCode::UNAUTHORIZED
    );

    new.reload().await.unwrap();
    assert_eq!(me_status(&new_app, &ended).await, StatusCode::UNAUTHORIZED);
    assert_eq!(me_status(&new_app, &created).await, StatusCode::OK);
}

/// Em desenvolvimento, os scripts locais criam sessão direto no Redis com a API
/// no ar.
#[tokio::test]
async fn in_development_an_unknown_session_is_looked_up_in_redis() {
    let (db, _lease) = crate::db::test_db().await;
    let me = user(&db, "Some").await;
    let development = booted(&db, true).await;
    let production = booted(&db, false).await;

    let script = old_format_session(&redis(&db), me.id, "sessao-de-script").await;
    assert_eq!(me_status(&development, &script).await, StatusCode::OK);
    assert_eq!(
        me_status(&production, &script).await,
        StatusCode::UNAUTHORIZED
    );
    let invented = User {
        id: me.id,
        cookie: "session=inventada".into(),
    };
    assert_eq!(
        me_status(&development, &invented).await,
        StatusCode::UNAUTHORIZED
    );
}
