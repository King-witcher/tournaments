//! Sair da comunidade e remover membro: quem remove quem, o que o inativo
//! perde (acesso, listas, inscrições abertas, convites) e a volta por
//! convite.

use axum::{Router, http::StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;
use crate::matches::{COPA, bracket, copa, single, start};

async fn remove(app: &Router, by: &User, who: &User) -> (StatusCode, Value) {
    delete(app, &format!("/tenants/quake-br/members/{}", who.id), by).await
}

async fn leave(app: &Router, who: &User) -> (StatusCode, Value) {
    delete(app, "/tenants/quake-br/me", who).await
}

const GONE: (StatusCode, Value) = (StatusCode::NO_CONTENT, Value::Null);

/// Os `user_id`s da lista de membros de `quake-br`, vista por `who`.
async fn member_ids(app: &Router, who: &User) -> Vec<i64> {
    let (status, list) = get(app, "/tenants/quake-br/members", who).await;
    assert_eq!(status, StatusCode::OK, "{list}");
    list.as_array()
        .unwrap()
        .iter()
        .map(|m| m["user_id"].as_i64().unwrap())
        .collect()
}

/// `who` já não vê `quake-br`: 404 como quem é de fora, e ela some da lista.
async fn assert_outside(app: &Router, who: &User) {
    assert_eq!(
        get(app, "/tenants/quake-br", who).await,
        (StatusCode::NOT_FOUND, err("tenant.not_found"))
    );
    assert_eq!(
        get(app, "/tenants/quake-br/tournaments", who).await,
        (StatusCode::NOT_FOUND, err("tenant.not_found"))
    );
    let (_, mine) = get(app, "/tenants", who).await;
    assert!(
        mine.as_array()
            .unwrap()
            .iter()
            .all(|t| t["slug"] != "quake-br"),
        "{mine}"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_owner_removes_organizers_and_players(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let olga = organizer(&app, &db, &ana, "Olga").await;

    assert_eq!(remove(&app, &ana, &olga).await, GONE);
    assert_eq!(remove(&app, &ana, &bia).await, GONE);
    assert_eq!(member_ids(&app, &ana).await, [ana.id]);
    assert_outside(&app, &olga).await;
    assert_outside(&app, &bia).await;
    // Quem já saiu não é mais membro.
    assert_eq!(
        remove(&app, &ana, &bia).await,
        (StatusCode::NOT_FOUND, err("member.not_found"))
    );
    let role = format!("/tenants/quake-br/members/{}/role", bia.id);
    assert_eq!(
        put(&app, &role, &ana, json!({ "role": "organizer" })).await,
        (StatusCode::NOT_FOUND, err("member.not_found"))
    );
    let who = sqlx::query!(
        "select user_id, removed_by, left_at is not null as \"left!\" from tenant_members
         where user_id = any($1) order by user_id",
        &[olga.id, bia.id]
    )
    .fetch_all(&db)
    .await
    .unwrap()
    .into_iter()
    .map(|m| (m.user_id, m.removed_by, m.left))
    .collect::<Vec<_>>();
    assert_eq!(
        who,
        [(bia.id, Some(ana.id), true), (olga.id, Some(ana.id), true)]
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn organizers_remove_only_players(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let olga = organizer(&app, &db, &ana, "Olga").await;
    let oto = organizer(&app, &db, &ana, "Oto").await;
    let caio = member(&app, &db, &ana, "Caio").await;

    let forbidden = (StatusCode::FORBIDDEN, err("tenant.forbidden"));
    assert_eq!(remove(&app, &olga, &oto).await, forbidden);
    assert_eq!(remove(&app, &bia, &caio).await, forbidden);
    assert_eq!(remove(&app, &bia, &olga).await, forbidden);
    // Ninguém remove o owner, nem ele mesmo; para sair, cada um sai.
    let owner = (StatusCode::CONFLICT, err("member.is_owner"));
    assert_eq!(remove(&app, &olga, &ana).await, owner);
    assert_eq!(
        remove(&app, &ana, &ana).await,
        (StatusCode::CONFLICT, err("member.remove_self"))
    );
    assert_eq!(
        remove(&app, &olga, &olga).await,
        (StatusCode::CONFLICT, err("member.remove_self"))
    );
    let eve = user(&db, "Eve").await;
    assert_eq!(
        remove(&app, &olga, &eve).await,
        (StatusCode::NOT_FOUND, err("member.not_found"))
    );

    assert_eq!(remove(&app, &olga, &bia).await, GONE);
    assert_eq!(
        member_ids(&app, &ana).await,
        [ana.id, olga.id, oto.id, caio.id]
    );
    assert_outside(&app, &bia).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn members_leave_but_the_owner_transfers_first(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let olga = organizer(&app, &db, &ana, "Olga").await;

    assert_eq!(
        leave(&app, &ana).await,
        (StatusCode::CONFLICT, err("member.owner_must_transfer"))
    );
    assert_eq!(leave(&app, &bia).await, GONE);
    assert_eq!(leave(&app, &olga).await, GONE);
    assert_outside(&app, &bia).await;
    assert_outside(&app, &olga).await;
    assert_eq!(member_ids(&app, &ana).await, [ana.id]);
    // Sair de novo: já é de fora.
    assert_eq!(
        leave(&app, &bia).await,
        (StatusCode::NOT_FOUND, err("tenant.not_found"))
    );
    let removed_by = sqlx::query_scalar!(
        "select removed_by from tenant_members where user_id = $1",
        bia.id
    )
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(removed_by, None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn leaving_drops_open_registrations_and_revokes_invites(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let olga = organizer(&app, &db, &ana, "Olga").await;
    // Em andamento, com a Olga na chave.
    start(&app, &ana, copa(single(3, 3)), &[&olga, &bia]).await;
    // Rascunho (o organizador inscreve), inscrição e check-in.
    for slug in ["rascunho", "aberta", "janela"] {
        create_tournament(&app, &ana, duel(slug)).await;
        let uri = format!("/tenants/quake-br/tournaments/{slug}/participants");
        let (status, body) = post(&app, &uri, &ana, json!({ "user_id": olga.id })).await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
    }
    publish(&app, &ana, "aberta").await;
    publish(&app, &ana, "janela").await;
    let (status, body) = post_empty(
        &app,
        "/tenants/quake-br/tournaments/janela/check-in/open",
        &ana,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // Convites dela: do tenant e link de inscrição.
    let tenant_link = invite(&app, &olga, "quake-br").await;
    let uri = "/tenants/quake-br/tournaments/aberta/invites";
    let (status, link) = post_empty(&app, uri, &olga).await;
    assert_eq!(status, StatusCode::CREATED, "{link}");

    assert_eq!(leave(&app, &olga).await, GONE);

    for slug in ["rascunho", "aberta", "janela"] {
        let uri = format!("/tenants/quake-br/tournaments/{slug}/participants");
        let (status, list) = get(&app, &uri, &ana).await;
        assert_eq!((status, list), (StatusCode::OK, json!([])), "{slug}");
    }
    for token in [&tenant_link, link["token"].as_str().unwrap()] {
        let (status, body) = call(
            &app,
            axum::http::Method::GET,
            &format!("/invites/{token}"),
            None,
            None,
        )
        .await;
        assert_eq!((status, body), (StatusCode::GONE, err("invite.revoked")));
    }
    // No torneio iniciado ela continua, marcada.
    let (_, list) = get(&app, &format!("{COPA}/participants"), &ana).await;
    let mine = list
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["user_id"] == olga.id)
        .unwrap();
    assert_eq!(mine["left"], true);
    let b = bracket(&app, &ana).await;
    assert_eq!(b["entrants"][0]["user_id"], olga.id);
    assert_eq!(b["entrants"][0]["left"], true);
    assert_eq!(b["entrants"][1]["left"], false);

    // Inativa, ninguém a inscreve.
    let uri = "/tenants/quake-br/tournaments/aberta/participants";
    assert_eq!(
        post(&app, uri, &ana, json!({ "user_id": olga.id })).await,
        (StatusCode::NOT_FOUND, err("member.not_found"))
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn undoing_the_start_drops_who_left(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let caio = member(&app, &db, &ana, "Caio").await;
    start(&app, &ana, copa(single(3, 3)), &[&ana, &bia, &caio]).await;
    assert_eq!(leave(&app, &bia).await, GONE);

    let (status, undone) = delete(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{undone}");
    let (_, list) = get(&app, &format!("{COPA}/participants"), &ana).await;
    let ids: Vec<_> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|p| &p["user_id"])
        .collect();
    assert_eq!(ids, [&json!(ana.id), &json!(caio.id)]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn coming_back_by_invite_reactivates_as_player(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let olga = organizer(&app, &db, &ana, "Olga").await;
    let (status, _) = patch(
        &app,
        "/tenants/quake-br/me",
        &olga,
        json!({ "nickname": "olga-qc" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(remove(&app, &ana, &olga).await, GONE);

    let back = join(&app, &olga, &invite(&app, &ana, "quake-br").await).await;
    assert_eq!(back["tenant"]["nickname"], "olga-qc");
    assert_eq!(back["tenant"]["role"], "player");
    let (status, seen) = get(&app, "/tenants/quake-br", &olga).await;
    assert_eq!((status, &seen["role"]), (StatusCode::OK, &json!("player")));
    let (_, list) = get(&app, "/tenants/quake-br/members", &ana).await;
    let olga_row = list
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["user_id"] == olga.id)
        .unwrap();
    assert_eq!(
        (&olga_row["nickname"], &olga_row["role"]),
        (&json!("olga-qc"), &json!("player"))
    );
    let row = sqlx::query!(
        "select left_at, removed_by from tenant_members where user_id = $1",
        olga.id
    )
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!((row.left_at, row.removed_by), (None, None));
}
