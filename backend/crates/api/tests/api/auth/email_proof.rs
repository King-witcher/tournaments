//! Conta em que o email é o único jeito de entrar: quem pegou a sessão não mexe
//! nos logins sem o código mandado ao email de agora. Sem isso, trocaria o único
//! login e o dono ficaria sem saída.

use api::mail::Email;
use axum::{Router, http::StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;

use super::{add_discord, app, authorize, discord_user, set_email, skip_cooldown};
use crate::common::{
    self, User, err, get, link_token, mailbox, post, post_empty, put, sessions, user,
};

const ADDRESS: &str = "q@outlook.com";

/// Conta só com email (sem senha nem provedor), com sessão.
async fn only_email(db: &PgPool) -> User {
    let me = user(db, "Quaker").await;
    set_email(db, me.id, ADDRESS).await;
    me
}

fn code_of(email: &Email) -> String {
    email
        .text
        .lines()
        .find(|line| line.len() == 6 && line.bytes().all(|b| b.is_ascii_digit()))
        .expect("email sem código")
        .to_owned()
}

fn other_code(code: &str) -> String {
    let n: u32 = code.parse().unwrap();
    format!("{:06}", (n + 1) % 1_000_000)
}

fn required() -> (StatusCode, Value) {
    (StatusCode::FORBIDDEN, err("auth.email_proof_required"))
}

fn rejected() -> (StatusCode, Value) {
    (StatusCode::UNPROCESSABLE_ENTITY, err("auth.code_rejected"))
}

/// Pede o código e devolve o email que saiu.
async fn request_proof(app: &Router, db: &PgPool, who: &User) -> Email {
    let (status, body) = post_empty(app, "/auth/email/proof", who).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    mailbox(db).next().await
}

async fn send_proof(app: &Router, who: &User, code: &str) -> (StatusCode, Value) {
    post(app, "/auth/email/proof/code", who, json!({ "code": code })).await
}

/// Prova o email para a sessão de `who`.
async fn prove(app: &Router, db: &PgPool, who: &User) {
    let email = request_proof(app, db, who).await;
    assert_eq!(
        send_proof(app, who, &code_of(&email)).await,
        (StatusCode::NO_CONTENT, Value::Null)
    );
}

#[tokio::test]
async fn changing_the_only_login_needs_the_code_of_the_current_address() {
    let (db, _lease) = crate::db::test_db().await;
    let app = common::app(db.clone());
    let me = only_email(&db).await;
    let change = json!({ "email": "novo@outlook.com" });

    // Só a sessão não basta, e nada sai para o endereço novo.
    assert_eq!(
        put(&app, "/auth/email", &me, change.clone()).await,
        required()
    );
    mailbox(&db).assert_empty().await;
    assert_eq!(
        get(&app, "/auth/email/proof", &me).await,
        (StatusCode::OK, json!({ "needed": true, "proven": false }))
    );

    // O código vai ao endereço de agora, com o que fazer se não foi a pessoa.
    let proof = request_proof(&app, &db, &me).await;
    assert_eq!(proof.to, ADDRESS);
    assert!(proof.text.contains("15 minutos"), "{}", proof.text);
    let wrong = other_code(&code_of(&proof));
    assert_eq!(send_proof(&app, &me, &wrong).await, rejected());
    assert_eq!(
        send_proof(&app, &me, &code_of(&proof)).await,
        (StatusCode::NO_CONTENT, Value::Null)
    );
    assert_eq!(
        get(&app, "/auth/email/proof", &me).await,
        (StatusCode::OK, json!({ "needed": true, "proven": true }))
    );

    // Só depois sai o email ao endereço novo, e a troca segue como sempre.
    let (status, body) = put(&app, "/auth/email", &me, change).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    let confirm = mailbox(&db).next().await;
    assert_eq!(confirm.to, "novo@outlook.com");
    let (status, body) = post(
        &app,
        "/auth/email/confirm",
        &me,
        json!({ "token": link_token(&confirm) }),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");

    // O aviso não manda entrar por um provedor que a conta não tem.
    let notice = mailbox(&db).next().await;
    assert_eq!(notice.to, ADDRESS);
    assert!(!notice.text.contains("Discord"), "{}", notice.text);
    assert!(notice.text.contains("este email"), "{}", notice.text);
}

/// Criar uma senha ou vincular um provedor tiraria a conta do "só email", e a
/// troca passaria sem o código.
#[tokio::test]
async fn the_ways_around_it_need_the_code_too() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone(), Some(discord_user(None)), true);
    let me = only_email(&db).await;

    let password = json!({ "password": "senha forte 1" });
    assert_eq!(
        post(&app, "/auth/password", &me, password.clone()).await,
        required()
    );
    assert_eq!(
        authorize(&app, "discord", Some(&me.cookie), None).await,
        required()
    );

    prove(&app, &db, &me).await;
    assert_eq!(
        authorize(&app, "discord", Some(&me.cookie), None).await.0,
        StatusCode::OK
    );
    assert_eq!(
        post(&app, "/auth/password", &me, password).await.0,
        StatusCode::CREATED
    );
}

/// O código prova quem o digitou: a sessão roubada não aproveita o dono ter
/// provado na dele.
#[tokio::test]
async fn the_proof_belongs_to_the_session_that_typed_the_code() {
    let (db, _lease) = crate::db::test_db().await;
    let app = common::app(db.clone());
    let owner = only_email(&db).await;
    let token = sessions(&db).create(owner.id, None).await.unwrap();
    let stolen = User {
        id: owner.id,
        cookie: format!("session={token}"),
    };

    prove(&app, &db, &owner).await;
    let change = json!({ "email": "atacante@outlook.com" });
    assert_eq!(put(&app, "/auth/email", &stolen, change).await, required());
}

#[tokio::test]
async fn five_wrong_codes_kill_the_code() {
    let (db, _lease) = crate::db::test_db().await;
    let app = common::app(db.clone());
    let me = only_email(&db).await;
    let proof = request_proof(&app, &db, &me).await;
    let code = code_of(&proof);
    let wrong = other_code(&code);

    for _ in 0..5 {
        assert_eq!(send_proof(&app, &me, &wrong).await, rejected());
    }
    assert_eq!(send_proof(&app, &me, &code).await, rejected());
    let change = json!({ "email": "novo@outlook.com" });
    assert_eq!(put(&app, "/auth/email", &me, change).await, required());
}

#[tokio::test]
async fn accounts_with_another_login_do_not_need_it() {
    let (db, _lease) = crate::db::test_db().await;
    let app = common::app(db.clone());
    let me = user(&db, "Discord").await;
    add_discord(&db, me.id).await;
    set_email(&db, me.id, ADDRESS).await;

    assert_eq!(
        get(&app, "/auth/email/proof", &me).await,
        (StatusCode::OK, json!({ "needed": false, "proven": false }))
    );
    assert_eq!(
        post_empty(&app, "/auth/email/proof", &me).await,
        (StatusCode::CONFLICT, err("auth.email_proof_not_needed"))
    );
    mailbox(&db).assert_empty().await;
    let change = json!({ "email": "novo@outlook.com" });
    assert_eq!(
        put(&app, "/auth/email", &me, change).await.0,
        StatusCode::ACCEPTED
    );
}

/// Pedir de novo fica na espera de 60 s, sem erro: é a mesma espera silenciosa
/// de entrar por email, que nenhuma resposta mostra.
#[tokio::test]
async fn asking_again_stays_quiet_inside_the_wait() {
    let (db, _lease) = crate::db::test_db().await;
    let app = common::app(db.clone());
    let me = only_email(&db).await;
    request_proof(&app, &db, &me).await;
    assert_eq!(
        post_empty(&app, "/auth/email/proof", &me).await.0,
        StatusCode::ACCEPTED
    );
    mailbox(&db).assert_empty().await;

    skip_cooldown(&db, ADDRESS).await;
    request_proof(&app, &db, &me).await;
}
