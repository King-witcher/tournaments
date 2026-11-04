//! Entrar por email: o link ou o código, o que vier primeiro, com a senha nova
//! opcional. Nada revela quem tem conta.

use api::mail::Email;
use axum::{
    Router,
    http::{Method, StatusCode},
};
use redis::Cmd;
use serde_json::{Value, json};
use sqlx::PgPool;

use super::{add_discord, send_json, session_count, set_email, signup, skip_cooldown};
use crate::common::{User, app, err, get, link_token, mailbox, patch, post, put, redis, user};

const PASSWORD: &str = "senha forte 1";
const NEW_PASSWORD: &str = "senha nova 22";
const ADDRESS: &str = "rapha@example.com";

async fn request(app: &Router, email: &str) -> StatusCode {
    let body = json!({ "email": email });
    let (status, body, _) = send_json(app, Method::POST, "/auth/email/login", None, body).await;
    assert_eq!(body, Value::Null, "a resposta não diz nada");
    status
}

/// Pede o email e o devolve. O cadastro acabou de mandar um para o endereço.
async fn requested(app: &Router, db: &PgPool, address: &str) -> Email {
    skip_cooldown(db, address).await;
    assert_eq!(request(app, address).await, StatusCode::ACCEPTED);
    mailbox(db).next().await
}

async fn by_link(
    app: &Router,
    email: &Email,
    password: Option<&str>,
) -> (StatusCode, Value, Option<String>) {
    let body = json!({ "token": link_token(email), "password": password });
    send_json(app, Method::POST, "/auth/email/login/confirm", None, body).await
}

async fn by_code(
    app: &Router,
    address: &str,
    code: &str,
    password: Option<&str>,
) -> (StatusCode, Value, Option<String>) {
    let body = json!({ "email": address, "code": code, "password": password });
    send_json(app, Method::POST, "/auth/email/login/code", None, body).await
}

async fn with_password(app: &Router, address: &str, password: &str) -> StatusCode {
    let body = json!({ "email": address, "password": password });
    send_json(app, Method::POST, "/auth/login", None, body)
        .await
        .0
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

/// A única resposta de um código que não entra.
fn rejected() -> (StatusCode, Value, Option<String>) {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        err("auth.code_rejected"),
        None,
    )
}

#[tokio::test]
async fn code_enters_and_the_link_stops_working() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;

    // Escrito de qualquer jeito que dê na forma canônica.
    skip_cooldown(&db, ADDRESS).await;
    assert_eq!(
        request(&app, " Rapha@Example.COM ").await,
        StatusCode::ACCEPTED
    );
    let email = mailbox(&db).next().await;
    assert_eq!(email.to, ADDRESS);
    assert!(
        email.text.contains("/pt/login/email/confirm#"),
        "{}",
        email.text
    );
    let code = code_of(&email);
    assert!(
        email.html.contains(&format!(">{code}</td>")),
        "código no HTML"
    );
    assert!(email.text.contains("15 minutos"), "{}", email.text);

    let (status, body, session) = by_code(&app, ADDRESS, &code, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["id"], me.id);
    assert!(session.is_some());

    // Sem senha nova, só abre a sessão: nada muda no resto da conta.
    assert_eq!(session_count(&app, &me).await, 2);
    assert_eq!(
        with_password(&app, ADDRESS, PASSWORD).await,
        StatusCode::OK,
        "a senha continua valendo"
    );
    mailbox(&db).assert_empty().await;

    // Valeu o código: nem ele nem o link valem mais.
    assert_eq!(by_code(&app, ADDRESS, &code, None).await, rejected());
    let (status, body, _) = by_link(&app, &email, None).await;
    assert_eq!((status, body), (StatusCode::GONE, err("auth.link_expired")));

    // E ao contrário: aberto o link, o código não vale.
    let email = requested(&app, &db, ADDRESS).await;
    assert_eq!(by_link(&app, &email, None).await.0, StatusCode::OK);
    assert_eq!(
        by_code(&app, ADDRESS, &code_of(&email), None).await,
        rejected()
    );
}

#[tokio::test]
async fn any_account_with_email_enters_and_in_its_language() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let discord = user(&db, "Discord").await;
    add_discord(&db, discord.id).await;
    set_email(&db, discord.id, "joaosilva@gmail.com").await;
    sqlx::query!("update users set locale = 'es'")
        .execute(&db)
        .await
        .unwrap();

    // Escrito de qualquer jeito que dê na forma canônica.
    assert_eq!(
        request(&app, "Joao.Silva+x@googlemail.com").await,
        StatusCode::ACCEPTED
    );
    let email = mailbox(&db).next().await;
    assert_eq!(email.to, "joaosilva@gmail.com");
    assert!(
        email.subject.contains("Entra en tu cuenta"),
        "{}",
        email.subject
    );
    assert!(
        email.text.contains("/es/login/email/confirm#"),
        "{}",
        email.text
    );

    let (status, body, session) = by_link(&app, &email, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["id"], discord.id);
    assert!(session.is_some());
}

#[tokio::test]
async fn nothing_says_who_has_an_account() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    // Conta sem email não recebe nada, como email sem conta.
    let discord = user(&db, "Discord").await;
    add_discord(&db, discord.id).await;
    for address in ["ninguem@outlook.com", "temp@mailinator.com"] {
        assert_eq!(
            request(&app, address).await,
            StatusCode::ACCEPTED,
            "{address}"
        );
        // Sem conta: a mesma resposta de um código errado, sem contagem.
        assert_eq!(by_code(&app, address, "123456", None).await, rejected());
    }
    mailbox(&db).assert_empty().await;

    // Com conta e sem pedido, ou com pedido e o código errado: igual.
    let me = signup(&app, &db, "Rapha", PASSWORD).await;
    assert_eq!(by_code(&app, ADDRESS, "123456", None).await, rejected());
    let email = requested(&app, &db, ADDRESS).await;
    let wrong = other_code(&code_of(&email));
    assert_eq!(by_code(&app, ADDRESS, &wrong, None).await, rejected());

    // Fora do formato, a recusa não depende de conta nenhuma.
    let (status, body, _) = send_json(
        &app,
        Method::POST,
        "/auth/email/login",
        None,
        json!({ "email": "Rapha" }),
    )
    .await;
    assert_eq!(
        (status, body),
        (StatusCode::UNPROCESSABLE_ENTITY, err("email.invalid"))
    );
    for code in ["12345", "1234567", "12a456", " 123456"] {
        let (status, body, _) = by_code(&app, ADDRESS, code, None).await;
        assert_eq!(
            (status, body),
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                err("request.invalid_body")
            ),
            "{code}"
        );
    }
    // O código de uma conta não serve a outra.
    let (status, _, _) = by_code(&app, "ninguem@outlook.com", &code_of(&email), None).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(session_count(&app, &me).await, 1);
}

#[tokio::test]
async fn five_wrong_codes_kill_the_code_but_not_the_link() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    signup(&app, &db, "Rapha", PASSWORD).await;
    let email = requested(&app, &db, ADDRESS).await;
    let code = code_of(&email);
    let wrong = other_code(&code);

    for _ in 0..5 {
        assert_eq!(by_code(&app, ADDRESS, &wrong, None).await, rejected());
    }
    // Esgotado, nem o certo entra.
    assert_eq!(by_code(&app, ADDRESS, &code, None).await, rejected());

    // O link continua valendo.
    assert_eq!(by_link(&app, &email, None).await.0, StatusCode::OK);
}

#[tokio::test]
async fn a_new_password_ends_the_other_sessions_and_warns() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;
    let email = requested(&app, &db, ADDRESS).await;
    let code = code_of(&email);

    // Senha fora do formato não gasta o link nem conta tentativa.
    for _ in 0..6 {
        let (status, body, _) = by_code(&app, ADDRESS, &code, Some("curta")).await;
        assert_eq!(
            (status, body),
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                err("auth.password_invalid")
            )
        );
    }
    let (status, body, _) = by_link(&app, &email, Some("curta")).await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("auth.password_invalid")
        )
    );

    let (status, body, session) = by_code(&app, ADDRESS, &code, Some(NEW_PASSWORD)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let fresh = User {
        id: me.id,
        cookie: session.unwrap(),
    };
    assert_eq!(get(&app, "/me", &me).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(session_count(&app, &fresh).await, 1);
    assert_eq!(
        with_password(&app, ADDRESS, PASSWORD).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        with_password(&app, ADDRESS, NEW_PASSWORD).await,
        StatusCode::OK
    );

    let notice = mailbox(&db).next().await;
    assert_eq!(notice.to, ADDRESS);
    assert!(
        notice.subject.contains("Sua conta tem uma senha nova"),
        "{}",
        notice.subject
    );
}

#[tokio::test]
async fn the_link_gives_a_password_to_an_account_without_one() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = user(&db, "Quaker").await;
    add_discord(&db, me.id).await;
    set_email(&db, me.id, "q@outlook.com").await;

    let email = requested(&app, &db, "q@outlook.com").await;
    let (status, body, session) = by_link(&app, &email, Some(NEW_PASSWORD)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["id"], me.id);
    let fresh = User {
        id: me.id,
        cookie: session.unwrap(),
    };
    let (_, logins) = get(&app, "/auth/logins", &fresh).await;
    assert_eq!(logins["password"], true);
    assert_eq!(
        with_password(&app, "q@outlook.com", NEW_PASSWORD).await,
        StatusCode::OK
    );
    mailbox(&db).next().await; // o aviso
}

#[tokio::test]
async fn wrong_passwords_of_before_do_not_block_the_new_one() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    signup(&app, &db, "Rapha", PASSWORD).await;
    for _ in 0..5 {
        with_password(&app, ADDRESS, "errada!!").await;
    }
    assert_eq!(
        with_password(&app, ADDRESS, PASSWORD).await,
        StatusCode::TOO_MANY_REQUESTS
    );

    let email = requested(&app, &db, ADDRESS).await;
    // Entrar sem senha nova não mexe na contagem, que é contra a senha de agora.
    assert_eq!(by_link(&app, &email, None).await.0, StatusCode::OK);
    assert_eq!(
        with_password(&app, ADDRESS, PASSWORD).await,
        StatusCode::TOO_MANY_REQUESTS
    );

    let email = requested(&app, &db, ADDRESS).await;
    assert_eq!(
        by_link(&app, &email, Some(NEW_PASSWORD)).await.0,
        StatusCode::OK
    );
    assert_eq!(
        with_password(&app, ADDRESS, NEW_PASSWORD).await,
        StatusCode::OK
    );
}

#[tokio::test]
async fn a_new_request_replaces_the_previous_and_limits_stay_quiet() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    signup(&app, &db, "Rapha", PASSWORD).await;
    // O email do cadastro não segura o de entrar: as esperas são separadas.
    assert_eq!(request(&app, ADDRESS).await, StatusCode::ACCEPTED);
    mailbox(&db).next().await;
    // Pedir de novo logo em seguida fica sem envio, calado.
    assert_eq!(request(&app, ADDRESS).await, StatusCode::ACCEPTED);
    mailbox(&db).assert_empty().await;

    let first = requested(&app, &db, ADDRESS).await;
    let second = requested(&app, &db, ADDRESS).await;

    let (status, body, _) = by_link(&app, &first, None).await;
    assert_eq!((status, body), (StatusCode::GONE, err("auth.link_expired")));
    if code_of(&first) != code_of(&second) {
        assert_eq!(
            by_code(&app, ADDRESS, &code_of(&first), None).await,
            rejected()
        );
    }
    // O pedido novo zera as tentativas.
    assert_eq!(
        by_code(&app, ADDRESS, &code_of(&second), None).await.0,
        StatusCode::OK
    );
}

/// Entrar por email fica fora da espera que o cadastro e a troca mostram: se um
/// pedido daqui marcasse a espera só de quem tem conta, o `POST /auth/signup` em
/// seguida (429 ou 202) diria quem tem.
#[tokio::test]
async fn asking_to_enter_does_not_show_in_the_signup() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    signup(&app, &db, "Rapha", PASSWORD).await;
    skip_cooldown(&db, ADDRESS).await;

    for address in [ADDRESS, "ninguem@outlook.com"] {
        assert_eq!(request(&app, address).await, StatusCode::ACCEPTED);
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let body = json!({ "display_name": "Sonda", "password": PASSWORD, "email": address });
        let (status, body, _) = send_json(&app, Method::POST, "/auth/signup", None, body).await;
        assert_eq!(status, StatusCode::ACCEPTED, "{address}: {body}");
    }
}

/// Os avisos sem código (`PUT` com o email de outra conta, cadastro com email em
/// uso) têm a espera deles: não seguram o email que entra, e o botão "Entrar por
/// email" do próprio aviso funciona. Sem isto, `PUT` a cada 60 s tiraria o login
/// por email da vítima.
#[tokio::test]
async fn notices_do_not_block_entering() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    signup(&app, &db, "Rapha", PASSWORD).await;
    skip_cooldown(&db, ADDRESS).await;
    let other = user(&db, "Outra").await;
    add_discord(&db, other.id).await;

    let change = json!({ "email": ADDRESS });
    assert_eq!(
        put(&app, "/auth/email", &other, change.clone()).await.0,
        StatusCode::ACCEPTED
    );
    let notice = mailbox(&db).next().await;
    assert!(notice.text.contains("/pt/login/email\n"), "{}", notice.text);

    assert_eq!(request(&app, ADDRESS).await, StatusCode::ACCEPTED);
    let email = mailbox(&db).next().await;
    assert_eq!(
        by_code(&app, ADDRESS, &code_of(&email), None).await.0,
        StatusCode::OK
    );

    // Continuam limitados, para não virar bombardeio.
    let (status, body) = put(&app, "/auth/email", &other, change).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "{body}");
    assert_eq!(body["error"], "email.too_soon");
}

/// As tentativas zeram a cada pedido, mas os erros da conta somam: pedir de novo
/// sem parar não dá mais palpites.
#[tokio::test]
async fn wrong_codes_add_up_across_requests() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    signup(&app, &db, "Rapha", PASSWORD).await;

    for _ in 0..4 {
        let email = requested(&app, &db, ADDRESS).await;
        let wrong = other_code(&code_of(&email));
        for _ in 0..5 {
            assert_eq!(by_code(&app, ADDRESS, &wrong, None).await, rejected());
        }
    }

    // Passou do teto: nem o código certo de um pedido novo entra.
    let email = requested(&app, &db, ADDRESS).await;
    assert_eq!(
        by_code(&app, ADDRESS, &code_of(&email), None).await,
        rejected()
    );
    // O link continua funcionando, e entrar zera os erros.
    assert_eq!(by_link(&app, &email, None).await.0, StatusCode::OK);
    let email = requested(&app, &db, ADDRESS).await;
    assert_eq!(
        by_code(&app, ADDRESS, &code_of(&email), None).await.0,
        StatusCode::OK
    );
}

/// A contagem é da conta, não de cada fluxo.
#[tokio::test]
async fn wrong_codes_of_the_email_change_count_for_entering() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;

    for n in 0..4 {
        let change = json!({ "email": format!("t{n}@outlook.com"), "current_password": PASSWORD });
        let (status, body) = put(&app, "/auth/email", &me, change).await;
        assert_eq!(status, StatusCode::ACCEPTED, "{body}");
        let wrong = other_code(&code_of(&mailbox(&db).next().await));
        for _ in 0..5 {
            let (status, body) =
                post(&app, "/auth/email/code", &me, json!({ "code": wrong })).await;
            assert_eq!(
                (status, body),
                (StatusCode::UNPROCESSABLE_ENTITY, err("auth.code_rejected"))
            );
        }
    }
    let email = requested(&app, &db, ADDRESS).await;
    assert_eq!(
        by_code(&app, ADDRESS, &code_of(&email), None).await,
        rejected()
    );
}

/// Se o Redis falha ao encerrar as sessões, a senha não pode ter mudado com as
/// sessões de quem a sabia ainda abertas.
#[tokio::test]
async fn a_new_password_is_only_saved_after_the_sessions_end() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;
    let email = requested(&app, &db, ADDRESS).await;
    let hash = || {
        sqlx::query_scalar!(
            "select password_hash from password_credentials where user_id = $1",
            me.id
        )
        .fetch_one(&db)
    };
    let before = hash().await.unwrap();

    // O índice das sessões com outro tipo: `delete_all_sessions` falha.
    let redis = redis(&db);
    let index = redis.key(format_args!("user_sessions:{}", me.id));
    redis.query::<()>(&Cmd::set(&index, "x")).await.unwrap();

    let (status, _, _) = by_link(&app, &email, Some(NEW_PASSWORD)).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(hash().await.unwrap(), before, "a senha fica");
}

/// Pedido criado logo depois de uma troca de email (o mesmo instante em que ela
/// derruba os pedidos) foi para o endereço antigo, e não entra.
#[tokio::test]
async fn a_request_for_the_old_address_does_not_enter() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;
    let by_old_link = requested(&app, &db, ADDRESS).await;
    set_email(&db, me.id, "novo@outlook.com").await;
    let (status, body, session) = by_link(&app, &by_old_link, None).await;
    assert_eq!((status, body), (StatusCode::GONE, err("auth.link_expired")));
    assert_eq!(session, None);

    set_email(&db, me.id, ADDRESS).await;
    let by_old_code = requested(&app, &db, ADDRESS).await;
    set_email(&db, me.id, "novo@outlook.com").await;
    assert_eq!(
        by_code(&app, "novo@outlook.com", &code_of(&by_old_code), None).await,
        rejected()
    );
}

#[tokio::test]
async fn it_lasts_fifteen_minutes() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;
    requested(&app, &db, ADDRESS).await;
    let redis = redis(&db);
    let pending = redis.key(format_args!("email_login_pending:{}", me.id));
    let ttl: i64 = redis.query(&Cmd::ttl(&pending)).await.unwrap();
    assert!((1..=15 * 60).contains(&ttl), "{ttl}");
}

#[tokio::test]
async fn failed_send_stays_silent_and_leaves_nothing() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;
    skip_cooldown(&db, ADDRESS).await;

    mailbox(&db).set_failing(true);
    // A resposta não muda: o pedido não diz quem tem conta.
    assert_eq!(request(&app, ADDRESS).await, StatusCode::ACCEPTED);
    let redis = redis(&db);
    let pending = redis.key(format_args!("email_login_pending:{}", me.id));
    let cooldown = redis.key(format_args!("email_quiet_cooldown:{ADDRESS}"));
    // O envio é em segundo plano: espera ele falhar e desfazer o pedido.
    let mut gone = false;
    for _ in 0..50 {
        let (pending, cooldown): (bool, bool) = redis
            .query_pipe(redis::pipe().exists(&pending).exists(&cooldown))
            .await
            .unwrap();
        if !pending && !cooldown {
            gone = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert!(gone, "o que não chegou não vale, e a espera de 60 s sai");
}

#[tokio::test]
async fn a_new_email_ends_the_pending_login() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;
    let old = requested(&app, &db, ADDRESS).await;

    // Foi para o email de antes, que pode ser de quem perdeu o acesso.
    let change = json!({ "email": "novo@outlook.com", "current_password": PASSWORD });
    assert_eq!(
        put(&app, "/auth/email", &me, change).await.0,
        StatusCode::ACCEPTED
    );
    let confirm = mailbox(&db).next().await;
    let body = json!({ "token": link_token(&confirm) });
    let (status, _, _) = send_json(&app, Method::POST, "/auth/email/confirm", None, body).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, body, _) = by_link(&app, &old, None).await;
    assert_eq!((status, body), (StatusCode::GONE, err("auth.link_expired")));
    // Nem o endereço de antes nem o novo aceitam o código de antes.
    assert_eq!(
        by_code(&app, ADDRESS, &code_of(&old), None).await,
        rejected()
    );
    assert_eq!(
        by_code(&app, "novo@outlook.com", &code_of(&old), None).await,
        rejected()
    );
}

#[tokio::test]
async fn a_new_password_ends_what_was_pending() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;
    let change_email = |email: &str, current: &str| {
        put(
            &app,
            "/auth/email",
            &me,
            json!({ "email": email, "current_password": current }),
        )
    };
    let confirm_email = |token: String| {
        send_json(
            &app,
            Method::POST,
            "/auth/email/confirm",
            None,
            json!({ "token": token }),
        )
    };

    // Trocada na conta: a troca de email e o de entrar pedidos antes caem.
    assert_eq!(
        change_email("novo@outlook.com", PASSWORD).await.0,
        StatusCode::ACCEPTED
    );
    let pending = link_token(&mailbox(&db).next().await);
    let login = requested(&app, &db, ADDRESS).await;
    let body = json!({ "current_password": PASSWORD, "password": NEW_PASSWORD });
    let (status, body) = patch(&app, "/auth/password", &me, body).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    mailbox(&db).next().await; // o aviso da senha trocada
    assert_eq!(confirm_email(pending).await.0, StatusCode::GONE);
    assert_eq!(by_link(&app, &login, None).await.0, StatusCode::GONE);

    // Definida ao entrar: a troca de email pedida antes cai.
    assert_eq!(
        change_email("outro@outlook.com", NEW_PASSWORD).await.0,
        StatusCode::ACCEPTED
    );
    let pending = link_token(&mailbox(&db).next().await);
    let login = requested(&app, &db, ADDRESS).await;
    let (status, _, session) = by_link(&app, &login, Some("outra senha 3")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(confirm_email(pending).await.0, StatusCode::GONE);
    let fresh = User {
        id: me.id,
        cookie: session.unwrap(),
    };
    let (_, logins) = get(&app, "/auth/logins", &fresh).await;
    assert_eq!(logins["email"], ADDRESS);
}

#[tokio::test]
async fn a_lost_email_refuses_what_was_pending() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = user(&db, "Quaker").await;
    add_discord(&db, me.id).await;
    set_email(&db, me.id, "q@outlook.com").await;
    let email = requested(&app, &db, "q@outlook.com").await;

    // Como a anonimização: o email sai sem passar pela rota que derruba o pedido.
    sqlx::query!("update users set email = null where id = $1", me.id)
        .execute(&db)
        .await
        .unwrap();
    let (status, body, session) = by_link(&app, &email, None).await;
    assert_eq!((status, body), (StatusCode::GONE, err("auth.link_expired")));
    assert_eq!(session, None);
}

#[tokio::test]
async fn entering_cancels_the_scheduled_deletion() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;
    skip_cooldown(&db, ADDRESS).await;
    let body = json!({ "current_password": PASSWORD });
    let (status, body) = post(&app, "/me/deletion", &me, body).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    mailbox(&db).next().await; // a data da exclusão

    let email = requested(&app, &db, ADDRESS).await;
    let (status, body, _) = by_code(&app, ADDRESS, &code_of(&email), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["deletion_cancelled"], true);
}
