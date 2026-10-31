//! Excluir a conta: o pedido (senha, owner, sessões, email), entrar de novo
//! cancela, e a execução pelo agendador (sai das comunidades, desiste dos
//! torneios em andamento, anonimiza e libera email e Discord).

use api::{auth::create_session, realtime};
use axum::{
    Router,
    http::{Method, StatusCode},
};
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use sqlx::PgPool;

use super::{add_discord, app, discord_user, do_login, email_of, send_json, signup, skip_cooldown};
use crate::common::{self, *};
use crate::matches::{COPA, bracket, copa, detail, single, start};
use crate::realtime::{outbox, tenant};

const PASSWORD: &str = "senha-boa-1";

/// Pede a exclusão da conta de `who` e devolve status, corpo e o Set-Cookie
/// da sessão (o que apaga o cookie).
async fn schedule(app: &Router, who: &User, body: Value) -> (StatusCode, Value, Option<String>) {
    send_json(app, Method::POST, "/me/deletion", Some(&who.cookie), body).await
}

/// Quando a exclusão da conta vence (nulo: nenhuma agendada).
async fn due(db: &PgPool, user_id: i64) -> Option<chrono::DateTime<Utc>> {
    sqlx::query_scalar!("select deletion_due_at from users where id = $1", user_id)
        .fetch_one(db)
        .await
        .unwrap()
}

#[tokio::test]
async fn scheduling_ends_every_session_and_emails_the_date() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone(), None, false);
    let rapha = signup(&app, &db, "Rapha", PASSWORD).await;
    let token = create_session(&redis(&db), rapha.id, None).await.unwrap();
    let other = User {
        id: rapha.id,
        cookie: format!("session={token}"),
    };

    // Com senha, a atual é obrigatória e confere.
    for body in [json!({}), json!({ "current_password": "outra-senha" })] {
        let (status, body, _) = schedule(&app, &rapha, body).await;
        assert_eq!(
            (status, body),
            (StatusCode::FORBIDDEN, err("auth.wrong_password"))
        );
    }
    assert_eq!(due(&db, rapha.id).await, None);

    skip_cooldown(&db, &email_of("Rapha")).await;
    let before = Utc::now();
    let (status, body, cookie) =
        schedule(&app, &rapha, json!({ "current_password": PASSWORD })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let scheduled: chrono::DateTime<Utc> =
        serde_json::from_value(body["scheduled_for"].clone()).unwrap();
    assert!(scheduled >= before + Duration::days(7) - Duration::seconds(5));
    assert!(scheduled <= Utc::now() + Duration::days(7));
    assert_eq!(due(&db, rapha.id).await, Some(scheduled));
    // O cookie sai, e as duas sessões acabaram.
    assert_eq!(cookie.as_deref(), Some("session="));
    for who in [&rapha, &other] {
        assert_eq!(
            common::get(&app, "/me", who).await,
            (StatusCode::UNAUTHORIZED, err("auth.unauthenticated"))
        );
    }
    let email = mailbox(&db).next().await;
    assert_eq!(email.to, email_of("Rapha"));
    assert!(
        email.subject.starts_with("Exclusão da conta agendada"),
        "{}",
        email.subject
    );
    let when = scheduled.format("%d/%m/%Y %H:%M").to_string();
    assert!(email.text.contains(&when), "{}", email.text);
    assert!(
        email.text.contains(&format!("{WEB}/pt/login")),
        "{}",
        email.text
    );
}

#[tokio::test]
async fn the_owner_transfers_or_deletes_the_community_first() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    // Sem senha (só o Discord), não pede a atual.
    let (status, body, _) = schedule(&app, &ana, json!({})).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("auth.owns_tenants"))
    );
    assert_eq!(due(&db, ana.id).await, None);

    let (status, body, _) = schedule(&app, &bia, json!({})).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // Ninguém passa a posse para quem pediu a exclusão da conta.
    let (status, body) = post(
        &app,
        "/tenants/quake-br/transfer-ownership",
        &ana,
        json!({ "user_id": bia.id, "confirm_slug": "quake-br" }),
    )
    .await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("member.deletion_scheduled"))
    );
}

#[tokio::test]
async fn signing_in_again_cancels_the_deletion() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone(), Some(discord_user(None)), true);
    let rapha = signup(&app, &db, "Rapha", PASSWORD).await;
    let (status, body, _) = schedule(&app, &rapha, json!({ "current_password": PASSWORD })).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let login = json!({ "email": "rapha@example.com", "password": PASSWORD });
    let (status, body, _) = send_json(&app, Method::POST, "/auth/login", None, login.clone()).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        (&body["id"], &body["deletion_cancelled"]),
        (&json!(rapha.id), &json!(true))
    );
    assert_eq!(due(&db, rapha.id).await, None);
    let (_, body, _) = send_json(&app, Method::POST, "/auth/login", None, login).await;
    assert_eq!(body["deletion_cancelled"], false);

    // Pelo Discord também, e o front leva à conta, que avisa.
    let (_, first) = do_login(&app).await;
    assert_eq!(first["deletion_cancelled"], false);
    let quaker = first["id"].as_i64().unwrap();
    sqlx::query!(
        "update users set deletion_due_at = now() + interval '7 days' where id = $1",
        quaker
    )
    .execute(&db)
    .await
    .unwrap();
    let (_, body) = do_login(&app).await;
    assert_eq!(body["deletion_cancelled"], true);
    assert_eq!(due(&db, quaker).await, None);
}

/// Inscrição de push direto no banco.
async fn subscribe_push(db: &PgPool, user_id: i64) {
    sqlx::query!(
        r#"insert into push_subscriptions (user_id, endpoint, p256dh, auth, locale)
           values ($1, 'https://push.example.com/rapha', $2, $3, 'pt')"#,
        user_id,
        "A".repeat(87),
        "B".repeat(22)
    )
    .execute(db)
    .await
    .unwrap();
}

#[tokio::test]
async fn the_due_deletion_anonymizes_and_keeps_the_history() {
    let (db, _lease) = crate::db::test_db().await;
    let state = state(db.clone());
    let app = app_from(state.clone());
    let ana = user(&db, "Ana").await;
    create(&app, &ana, "quake-br").await;
    create(&app, &ana, "outra").await;
    // Rapha: senha, email, Discord, push, organizer em quake-br (com um
    // convite aberto) e ex-membro de outra.
    let rapha = super::signup(&app, &db, "Rapha", PASSWORD).await;
    add_discord(&db, rapha.id).await;
    subscribe_push(&db, rapha.id).await;
    sqlx::query(
        r#"insert into notifications (user_id, notice)
           values ($1, '{"code": "check_in.opened", "params": {"tournament": "Copa"}, "path": "/outra/copa"}')"#,
    )
    .bind(rapha.id)
    .execute(&db)
    .await
    .unwrap();
    join(&app, &rapha, &invite(&app, &ana, "outra").await).await;
    let (status, _) = delete(&app, "/tenants/outra/me", &rapha).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    join(&app, &rapha, &invite(&app, &ana, "quake-br").await).await;
    let uri = format!("/tenants/quake-br/members/{}/role", rapha.id);
    let (status, _) = put(&app, &uri, &ana, json!({ "role": "organizer" })).await;
    assert_eq!(status, StatusCode::OK);
    let link = invite(&app, &rapha, "quake-br").await;
    let bia = member(&app, &db, &ana, "Bia").await;
    // Em andamento com a Bia, e inscrito num que não começou.
    start(&app, &ana, copa(single(3, 3)), &[&rapha, &bia]).await;
    create_tournament(&app, &ana, duel("aberta")).await;
    publish(&app, &ana, "aberta").await;
    register(&app, "aberta", &[&rapha]).await;

    skip_cooldown(&db, &email_of("Rapha")).await;
    let (status, body, _) = schedule(&app, &rapha, json!({ "current_password": PASSWORD })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(mailbox(&db).next().await.subject.starts_with("Exclusão"));
    // Antes da data, nada.
    let tick = realtime::tick(&state, Utc::now()).await.unwrap().unwrap();
    assert_eq!(tick.deleted_accounts, 0);

    crate::realtime::clear(&db).await;
    let later = Utc::now() + Duration::days(7) + Duration::minutes(1);
    let tick = realtime::tick(&state, later).await.unwrap().unwrap();
    assert_eq!(tick.deleted_accounts, 1);
    let tick = realtime::tick(&state, later).await.unwrap().unwrap();
    assert_eq!(tick.deleted_accounts, 0);

    // Anonimizada: sem nome, avatar, email, logins, push nem notificações.
    let row = sqlx::query!(
        r#"select display_name, avatar_url, email, deletion_due_at, deleted_at is not null as "deleted!",
                  (select count(*) from user_identities where user_id = u.id) as "identities!",
                  (select count(*) from password_credentials where user_id = u.id) as "passwords!",
                  (select count(*) from push_subscriptions where user_id = u.id) as "push!",
                  (select count(*) from notifications where user_id = u.id) as "notifications!"
           from users u where id = $1"#,
        rapha.id
    )
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(
        (
            row.display_name.as_str(),
            row.avatar_url,
            row.email,
            row.deletion_due_at
        ),
        ("", None, None, None)
    );
    assert!(row.deleted);
    assert_eq!(
        (row.identities, row.passwords, row.push, row.notifications),
        (0, 0, 0, 0)
    );
    // Inativa nas duas comunidades, com o apelido vazio.
    let members = sqlx::query!(
        r#"select nickname, left_at is not null as "left!" from tenant_members where user_id = $1"#,
        rapha.id
    )
    .fetch_all(&db)
    .await
    .unwrap();
    assert_eq!(members.len(), 2);
    assert!(members.iter().all(|m| m.nickname.is_empty() && m.left));
    let events = outbox(&db).await;
    assert!(events.contains(&tenant("quake-br")));
    assert!(events.contains(&tenant("outra")));
    assert!(
        events.contains(&json!({ "kind": "access_lost", "user": rapha.id, "tenant": "quake-br" }))
    );

    // O convite dela saiu, e a inscrição do que não começou também.
    let (status, body) = call(&app, Method::GET, &format!("/invites/{link}"), None, None).await;
    assert_eq!((status, body), (StatusCode::GONE, err("invite.revoked")));
    let uri = "/tenants/quake-br/tournaments/aberta/participants";
    assert_eq!(
        common::get(&app, uri, &ana).await,
        (StatusCode::OK, json!([]))
    );
    // No em andamento, desistiu: W.O. para a Bia, e o nome é o marcador.
    let d = detail(&app, &ana, "wb-1-1").await;
    assert_eq!(d["summary"]["result"]["confirmed_via"], "withdrawal");
    let b = bracket(&app, &ana).await;
    let entrant = &b["entrants"][0];
    let player = &entrant["members"][0];
    assert_eq!(
        (
            &player["user_id"],
            &player["nickname"],
            &player["left"],
            &entrant["withdrawn"]
        ),
        (&json!(rapha.id), &json!(""), &json!(true), &json!(true))
    );
    let (_, list) = common::get(&app, &format!("{COPA}/participants"), &ana).await;
    let mine = list
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["user_id"] == rapha.id)
        .unwrap();
    assert_eq!(
        (&mine["nickname"], &mine["left"]),
        (&json!(""), &json!(true))
    );

    // O email e o Discord ficam livres para outra conta.
    skip_cooldown(&db, &email_of("Rapha")).await;
    let again = super::signup(&app, &db, "Rapha", PASSWORD).await;
    assert_ne!(again.id, rapha.id);
    sqlx::query!(
        "insert into user_identities (user_id, provider, provider_user_id) values ($1, 'discord', $2)",
        again.id,
        format!("discord-{}", rapha.id)
    )
    .execute(&db)
    .await
    .unwrap();
}
