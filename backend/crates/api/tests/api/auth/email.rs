//! Pôr, trocar ou tirar o email da conta logada; pôr e trocar, confirmados
//! pelo link ou pelo código.

use api::mail::Email;
use axum::{
    Router,
    http::{Method, StatusCode},
};
use serde_json::{Value, json};

use super::{add_discord, send_json, set_email, signup, skip_cooldown};
use crate::common::{User, app, call, delete, err, get, link_token, mailbox, post, put, user};

const PASSWORD: &str = "senha forte 1";

async fn confirm(app: &Router, token: &str) -> (StatusCode, Value) {
    let body = json!({ "token": token });
    let (status, body, _) = send_json(app, Method::POST, "/auth/email/confirm", None, body).await;
    (status, body)
}

async fn email_of(app: &Router, who: &User) -> Value {
    get(app, "/auth/logins", who).await.1["email"].clone()
}

/// O código do email: a linha só com os 6 dígitos.
fn code_of(email: &Email) -> String {
    email
        .text
        .lines()
        .find(|line| line.len() == 6 && line.bytes().all(|b| b.is_ascii_digit()))
        .expect("email sem código")
        .to_owned()
}

/// Um código de 6 dígitos que não é `code`.
fn other_code(code: &str) -> String {
    let n: u32 = code.parse().unwrap();
    format!("{:06}", (n + 1) % 1_000_000)
}

async fn send_code(app: &Router, who: &User, code: &str) -> (StatusCode, Value) {
    post(app, "/auth/email/code", who, json!({ "code": code })).await
}

/// A única resposta de um código que não confirma.
fn rejected() -> (StatusCode, Value) {
    (StatusCode::UNPROCESSABLE_ENTITY, err("auth.code_rejected"))
}

#[tokio::test]
async fn code_confirms_the_email_and_the_link_stops_working() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = user(&db, "Quaker").await;
    add_discord(&db, me.id).await;

    let change = json!({ "email": "q@outlook.com" });
    assert_eq!(
        put(&app, "/auth/email", &me, change).await.0,
        StatusCode::ACCEPTED
    );
    let email = mailbox(&db).next().await;
    let code = code_of(&email);
    assert!(
        email.html.contains(&format!(">{code}</td>")),
        "código no HTML"
    );
    assert_eq!(email_of(&app, &me).await, Value::Null, "ainda não vale");

    assert_eq!(
        send_code(&app, &me, &code).await,
        (StatusCode::NO_CONTENT, Value::Null)
    );
    assert_eq!(email_of(&app, &me).await, "q@outlook.com");
    assert_eq!(send_code(&app, &me, &code).await, rejected());
    assert_eq!(
        confirm(&app, &link_token(&email)).await,
        (StatusCode::GONE, err("auth.link_expired"))
    );

    // E ao contrário: aberto o link, o código não vale. Quem tinha email recebe
    // o aviso, como pelo link.
    let change = json!({ "email": "z@outlook.com" });
    assert_eq!(
        put(&app, "/auth/email", &me, change).await.0,
        StatusCode::ACCEPTED
    );
    let email = mailbox(&db).next().await;
    assert_eq!(
        confirm(&app, &link_token(&email)).await.0,
        StatusCode::NO_CONTENT
    );
    mailbox(&db).next().await; // o aviso ao email de antes
    assert_eq!(send_code(&app, &me, &code_of(&email)).await, rejected());
}

#[tokio::test]
async fn five_wrong_codes_kill_the_code_but_not_the_link() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = user(&db, "Quaker").await;
    add_discord(&db, me.id).await;
    let change = json!({ "email": "q@outlook.com" });
    put(&app, "/auth/email", &me, change).await;
    let email = mailbox(&db).next().await;
    let code = code_of(&email);
    let wrong = other_code(&code);

    for _ in 0..5 {
        assert_eq!(send_code(&app, &me, &wrong).await, rejected());
    }
    assert_eq!(send_code(&app, &me, &code).await, rejected());
    assert_eq!(
        confirm(&app, &link_token(&email)).await.0,
        StatusCode::NO_CONTENT
    );
}

/// Pôr o email de outra conta não deixa pedido, e o código não pode revelar.
#[tokio::test]
async fn code_answers_the_same_for_the_email_of_another_account() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    signup(&app, &db, "Rapha", PASSWORD).await;
    skip_cooldown(&db, "rapha@example.com").await;
    let me = user(&db, "Quaker").await;
    add_discord(&db, me.id).await;

    assert_eq!(send_code(&app, &me, "123456").await, rejected());
    let taken = json!({ "email": "rapha@example.com" });
    assert_eq!(
        put(&app, "/auth/email", &me, taken).await.0,
        StatusCode::ACCEPTED
    );
    mailbox(&db).next().await; // o aviso ao dono
    assert_eq!(send_code(&app, &me, "123456").await, rejected());

    let free = json!({ "email": "q@outlook.com" });
    assert_eq!(
        put(&app, "/auth/email", &me, free).await.0,
        StatusCode::ACCEPTED
    );
    let wrong = other_code(&code_of(&mailbox(&db).next().await));
    assert_eq!(send_code(&app, &me, &wrong).await, rejected());
}

#[tokio::test]
async fn code_needs_the_session_and_six_digits() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = user(&db, "Quaker").await;
    let body = json!({ "code": "123456" });
    let (status, body, _) = send_json(&app, Method::POST, "/auth/email/code", None, body).await;
    assert_eq!(
        (status, body),
        (StatusCode::UNAUTHORIZED, err("auth.unauthenticated"))
    );
    for code in ["12345", "1234567", "12a456"] {
        assert_eq!(
            send_code(&app, &me, code).await,
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                err("request.invalid_body")
            ),
            "{code}"
        );
    }
}

/// Outra sessão da mesma conta, como a que alguém roubou.
async fn another_session(db: &sqlx::PgPool, id: i64) -> User {
    let token = crate::common::sessions(db).create(id, None).await.unwrap();
    User {
        id,
        cookie: format!("session={token}"),
    }
}

/// Com a sessão roubada, pede-se o email do atacante; o dono encerra a sessão, e
/// o pedido morre com ela, sem depender de quem abre o link.
#[tokio::test]
async fn ending_the_session_kills_the_change_it_asked_for() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    for (n, end) in ["logout", "others", "one"].into_iter().enumerate() {
        for by_link in [true, false] {
            let owner = user(&db, "Dono").await;
            add_discord(&db, owner.id).await;
            let stolen = another_session(&db, owner.id).await;
            let target = json!({ "email": format!("atacante{n}{by_link}@outlook.com") });
            assert_eq!(
                put(&app, "/auth/email", &stolen, target).await.0,
                StatusCode::ACCEPTED
            );
            let email = mailbox(&db).next().await;

            match end {
                "logout" => {
                    call(&app, Method::POST, "/auth/logout", Some(&stolen), None).await;
                }
                "others" => {
                    delete(&app, "/auth/sessions", &owner).await;
                }
                _ => {
                    let (_, sessions) = get(&app, "/auth/sessions", &owner).await;
                    let id = sessions
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|s| s["current"] == false)
                        .unwrap()["id"]
                        .as_str()
                        .unwrap()
                        .to_owned();
                    delete(&app, &format!("/auth/sessions/{id}"), &owner).await;
                }
            }

            if by_link {
                assert_eq!(
                    confirm(&app, &link_token(&email)).await,
                    (StatusCode::GONE, err("auth.link_expired")),
                    "{end}"
                );
            } else {
                let keep = another_session(&db, owner.id).await;
                assert_eq!(
                    send_code(&app, &keep, &code_of(&email)).await,
                    rejected(),
                    "{end}"
                );
            }
            let fresh = another_session(&db, owner.id).await;
            assert_eq!(email_of(&app, &fresh).await, Value::Null, "{end}");
        }
    }
}

/// O link abre em outro navegador, e o código em outra sessão: só a existência da
/// sessão que pediu é conferida.
#[tokio::test]
async fn the_code_works_from_another_session_while_the_first_lives() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = user(&db, "Quaker").await;
    add_discord(&db, me.id).await;
    let change = json!({ "email": "q@outlook.com" });
    put(&app, "/auth/email", &me, change).await;
    let email = mailbox(&db).next().await;
    let other = another_session(&db, me.id).await;
    assert_eq!(
        send_code(&app, &other, &code_of(&email)).await.0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(email_of(&app, &me).await, "q@outlook.com");
}

/// A espera de 60 s é por endereço: sem isto, uma conta gasta o teto do dia
/// mudando de endereço.
#[tokio::test]
async fn one_account_does_not_spend_the_daily_limit_alone() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = user(&db, "Quaker").await;
    add_discord(&db, me.id).await;
    let change = |n: i32| json!({ "email": format!("a{n}@outlook.com") });

    for n in 0..5 {
        let (status, body) = put(&app, "/auth/email", &me, change(n)).await;
        assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    }
    let (status, body) = put(&app, "/auth/email", &me, change(5)).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "{body}");
    assert_eq!(body["error"], "email.account_limited");
    assert!(body["params"]["minutes"].as_i64().unwrap() >= 1);
    for _ in 0..5 {
        mailbox(&db).next().await;
    }
    mailbox(&db).assert_empty().await;

    // Outra conta segue.
    let other = user(&db, "Outra").await;
    add_discord(&db, other.id).await;
    assert_eq!(
        put(&app, "/auth/email", &other, change(6)).await.0,
        StatusCode::ACCEPTED
    );
}

/// Com um pedido anterior, pôr o email de outra conta não pode deixar o código
/// de antes valendo: quem o tem descobriria que o endereço já tem dono.
#[tokio::test]
async fn email_of_another_account_ends_the_previous_request() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    signup(&app, &db, "Rapha", PASSWORD).await;
    skip_cooldown(&db, "rapha@example.com").await;
    let me = user(&db, "Quaker").await;
    add_discord(&db, me.id).await;

    let first = json!({ "email": "meu@outlook.com" });
    put(&app, "/auth/email", &me, first).await;
    let previous = mailbox(&db).next().await;
    let taken = json!({ "email": "rapha@example.com" });
    assert_eq!(
        put(&app, "/auth/email", &me, taken).await.0,
        StatusCode::ACCEPTED
    );
    mailbox(&db).next().await; // o aviso ao dono

    assert_eq!(send_code(&app, &me, &code_of(&previous)).await, rejected());
    assert_eq!(
        confirm(&app, &link_token(&previous)).await,
        (StatusCode::GONE, err("auth.link_expired"))
    );
    assert_eq!(email_of(&app, &me).await, Value::Null);
}

/// O email é um jeito de entrar: sem senha nem provedor, é o último.
#[tokio::test]
async fn the_last_login_email_stays() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = user(&db, "Quaker").await;
    set_email(&db, me.id, "q@outlook.com").await;
    assert_eq!(
        delete(&app, "/auth/email", &me).await,
        (StatusCode::CONFLICT, err("auth.last_login"))
    );
    assert_eq!(email_of(&app, &me).await, "q@outlook.com");
}

#[tokio::test]
async fn new_email_counts_only_after_the_link() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;

    let change = json!({ "email": "Rapha.Q@Gmail.com", "current_password": PASSWORD });
    let (status, body) = put(&app, "/auth/email", &me, change).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    assert_eq!(body, json!({ "email": "raphaq@gmail.com" }));
    assert_eq!(
        email_of(&app, &me).await,
        "rapha@example.com",
        "até o link, fica o de antes"
    );

    let email = mailbox(&db).next().await;
    assert_eq!(email.to, "raphaq@gmail.com");
    assert!(email.text.contains("/pt/email/confirm#"), "{}", email.text);
    // O link não pede login: pode abrir em outro aparelho.
    assert_eq!(
        confirm(&app, &link_token(&email)).await.0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(email_of(&app, &me).await, "raphaq@gmail.com");
    assert_eq!(
        confirm(&app, &link_token(&email)).await,
        (StatusCode::GONE, err("auth.link_expired"))
    );
}

#[tokio::test]
async fn password_accounts_confirm_with_the_current_password() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;
    for change in [
        json!({ "email": "novo@outlook.com" }),
        json!({ "email": "novo@outlook.com", "current_password": "errada!!" }),
    ] {
        let (status, body) = put(&app, "/auth/email", &me, change).await;
        assert_eq!(
            (status, body),
            (StatusCode::FORBIDDEN, err("auth.wrong_password"))
        );
    }

    for (address, status, code) in [
        (
            "Rapha+x@example.com",
            StatusCode::CONFLICT,
            "auth.email_unchanged",
        ),
        (
            "x@mailinator.com",
            StatusCode::UNPROCESSABLE_ENTITY,
            "email.disposable",
        ),
    ] {
        let change = json!({ "email": address, "current_password": PASSWORD });
        let (got, body) = put(&app, "/auth/email", &me, change).await;
        assert_eq!((got, body), (status, err(code)), "{address}");
    }
    mailbox(&db).assert_empty().await;
}

#[tokio::test]
async fn discord_account_sets_email_without_password() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = user(&db, "Quaker").await;
    add_discord(&db, me.id).await;
    assert_eq!(email_of(&app, &me).await, Value::Null);

    let (status, _) = put(
        &app,
        "/auth/email",
        &me,
        json!({ "email": "q@outlook.com" }),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    confirm(&app, &link_token(&mailbox(&db).next().await)).await;
    assert_eq!(email_of(&app, &me).await, "q@outlook.com");

    // Outra conta que confirmou o mesmo email antes leva a vez.
    let other = user(&db, "Outra").await;
    let (status, _) = put(
        &app,
        "/auth/email",
        &other,
        json!({ "email": "o@outlook.com" }),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let pending = mailbox(&db).next().await;
    set_email(&db, me.id, "o@outlook.com").await;
    assert_eq!(
        confirm(&app, &link_token(&pending)).await,
        (StatusCode::CONFLICT, err("auth.email_taken"))
    );

    let (status, _) = put(
        &app,
        "/auth/email",
        &me,
        json!({ "email": "z@outlook.com" }),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let unauthenticated = send_json(
        &app,
        Method::PUT,
        "/auth/email",
        None,
        json!({ "email": "z@outlook.com" }),
    )
    .await;
    assert_eq!(unauthenticated.0, StatusCode::UNAUTHORIZED);
}

/// A conta sem senha troca o email só com a sessão: a resposta não pode dizer
/// quem tem conta.
#[tokio::test]
async fn email_of_another_account_answers_the_same() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let other = signup(&app, &db, "Rapha", PASSWORD).await;
    skip_cooldown(&db, "rapha@example.com").await;
    let me = user(&db, "Quaker").await;
    add_discord(&db, me.id).await;

    let change = json!({ "email": "Rapha@Example.com" });
    let (status, body) = put(&app, "/auth/email", &me, change.clone()).await;
    assert_eq!(
        (status, body),
        (
            StatusCode::ACCEPTED,
            json!({ "email": "rapha@example.com" })
        )
    );
    // O dono fica sabendo, e não sai link.
    let notice = mailbox(&db).next().await;
    assert_eq!(notice.to, "rapha@example.com");
    assert!(notice.text.contains("já é da sua conta"), "{}", notice.text);
    assert!(
        !notice.text.contains("/email/confirm"),
        "sem link: {}",
        notice.text
    );

    // Pedir de novo esbarra na mesma espera de um email livre.
    let (status, body) = put(&app, "/auth/email", &me, change).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["error"], "email.too_soon");
    assert_eq!(email_of(&app, &me).await, Value::Null);
    assert_eq!(email_of(&app, &other).await, "rapha@example.com");
}

#[tokio::test]
async fn only_accounts_without_password_remove_the_email() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    // Com senha, ela entra com o email.
    let rapha = signup(&app, &db, "Rapha", PASSWORD).await;
    assert_eq!(
        delete(&app, "/auth/email", &rapha).await,
        (StatusCode::CONFLICT, err("auth.password_needs_email"))
    );
    assert_eq!(email_of(&app, &rapha).await, "rapha@example.com");

    let me = user(&db, "Quaker").await;
    add_discord(&db, me.id).await;
    assert_eq!(
        delete(&app, "/auth/email", &me).await,
        (StatusCode::CONFLICT, err("auth.no_email"))
    );

    // Com um link de troca esperando, tirar o email derruba o link.
    set_email(&db, me.id, "q@outlook.com").await;
    let change = json!({ "email": "novo@outlook.com" });
    assert_eq!(
        put(&app, "/auth/email", &me, change).await.0,
        StatusCode::ACCEPTED
    );
    let pending = mailbox(&db).next().await;
    let (status, body) = delete(&app, "/auth/email", &me).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body,
        json!({ "discord": true, "google": false, "password": false, "email": null })
    );
    assert_eq!(
        confirm(&app, &link_token(&pending)).await,
        (StatusCode::GONE, err("auth.link_expired"))
    );
    assert_eq!(email_of(&app, &me).await, Value::Null);

    let unauthenticated = call(&app, Method::DELETE, "/auth/email", None, None).await;
    assert_eq!(unauthenticated.0, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn failed_send_leaves_no_link_and_allows_a_retry_right_away() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;
    let change = json!({ "email": "novo@outlook.com", "current_password": PASSWORD });

    mailbox(&db).set_failing(true);
    assert_eq!(
        put(&app, "/auth/email", &me, change.clone()).await,
        (StatusCode::BAD_GATEWAY, err("email.send_failed"))
    );
    let redis = crate::common::redis(&db);
    let link = redis.key(format_args!("email_change_user:{}", me.id));
    let kept: bool = redis.query(&redis::Cmd::exists(link)).await.unwrap();
    assert!(!kept, "o link que não chegou não fica valendo");

    mailbox(&db).set_failing(false);
    let (status, body) = put(&app, "/auth/email", &me, change).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    let email = mailbox(&db).next().await;
    assert_eq!(
        confirm(&app, &link_token(&email)).await.0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(email_of(&app, &me).await, "novo@outlook.com");
}
