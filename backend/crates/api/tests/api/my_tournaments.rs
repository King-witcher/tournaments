//! `GET /me/tournaments`: os torneios de quem vê em todas as comunidades, com
//! a situação dele e a próxima partida; e `.../mine`, a mesma entrada de um
//! torneio só.

use axum::{
    Router,
    http::{Method, StatusCode},
};
use serde_json::{Value, json};

use crate::common::*;
use crate::matches::{
    COPA, bracket, confirm, copa, game, play, report, side_player, single, start,
};

async fn mine(app: &Router, who: &User) -> Value {
    let (status, body) = get(app, "/me/tournaments", who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["registered"].clone()
}

/// A entrada de `who` no torneio `slug` de `quake-br` (`.../mine`), conferida
/// com a da lista: a rota de um torneio só traz o mesmo item.
async fn entry(app: &Router, who: &User, slug: &str) -> Value {
    let uri = format!("/tenants/quake-br/tournaments/{slug}/mine");
    let (status, one) = get(app, &uri, who).await;
    assert_eq!(status, StatusCode::OK, "{one}");
    let list = mine(app, who).await;
    let listed = list
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["tenant_slug"] == "quake-br" && t["tournament"]["slug"] == slug)
        .unwrap_or_else(|| panic!("{slug} fora da lista: {list}"));
    assert_eq!(&one, listed);
    one
}

/// `comunidade/torneio` dos abertos à inscrição de `who`, na ordem.
async fn open(app: &Router, who: &User) -> Vec<String> {
    let (status, body) = get(app, "/me/tournaments", who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["open"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| {
            format!(
                "{}/{}",
                t["tenant_slug"].as_str().unwrap(),
                t["tournament"]["slug"].as_str().unwrap()
            )
        })
        .collect()
}

/// `(comunidade/torneio, situação)` de cada item, na ordem.
fn situations(list: &Value) -> Vec<(String, String)> {
    list.as_array()
        .unwrap()
        .iter()
        .map(|t| {
            (
                format!(
                    "{}/{}",
                    t["tenant_slug"].as_str().unwrap(),
                    t["tournament"]["slug"].as_str().unwrap()
                ),
                t["situation"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

#[tokio::test]
async fn requires_login() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let (status, body) = call(&app, Method::GET, "/me/tournaments", None, None).await;
    assert_eq!(
        (status, body),
        (StatusCode::UNAUTHORIZED, err("auth.unauthenticated"))
    );
}

#[tokio::test]
async fn open_lists_what_the_button_would_accept() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let caio = member(&app, &db, &ana, "Caio").await;
    // Rascunho e só por link ficam de fora, também para Ana, que vê os dois.
    create_tournament(&app, &ana, duel("rascunho")).await;
    let invite_only = with(duel("fechado"), json!({ "admission": "invite_only" }));
    create_tournament(&app, &ana, invite_only).await;
    publish(&app, &ana, "fechado").await;
    // Pelo início, e o sem data por último, seja qual for a ordem de criação.
    for (slug, starts_at) in [
        ("sem-data", Value::Null),
        ("depois", json!(FAR_START)),
        ("antes", json!("2098-01-01T20:00:00Z")),
    ] {
        let body = with(duel(slug), json!({ "starts_at": starts_at }));
        create_tournament(&app, &ana, body).await;
        publish(&app, &ana, slug).await;
    }
    let all = ["quake-br/antes", "quake-br/depois", "quake-br/sem-data"];
    assert_eq!(open(&app, &ana).await, all);
    assert_eq!(open(&app, &bia).await, all);

    // Quem já se inscreveu sai dos abertos e entra nos dele; o outro continua.
    register(&app, "antes", &[&bia]).await;
    assert_eq!(
        open(&app, &bia).await,
        ["quake-br/depois", "quake-br/sem-data"]
    );
    assert_eq!(open(&app, &caio).await, all);

    // Sem vaga, sai dos abertos de quem não entrou.
    let two = with(duel("lotado"), json!({ "max_participants": 2 }));
    create_tournament(&app, &ana, two).await;
    publish(&app, &ana, "lotado").await;
    assert!(
        open(&app, &caio)
            .await
            .contains(&"quake-br/lotado".to_owned())
    );
    register(&app, "lotado", &[&ana, &bia]).await;
    assert!(
        !open(&app, &caio)
            .await
            .contains(&"quake-br/lotado".to_owned())
    );
}

#[tokio::test]
async fn situation_before_the_bracket() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let caio = member(&app, &db, &ana, "Caio").await;
    // Torneio em que Bia não está: fica de fora.
    create_tournament(&app, &ana, duel("outro")).await;
    publish(&app, &ana, "outro").await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&bia, &caio, &ana]).await;
    // Outra comunidade de Bia, com um torneio em que ela está.
    create(&app, &bia, "argentina").await;
    let (status, body) = post(
        &app,
        "/tenants/argentina/tournaments",
        &bia,
        with(duel("liga"), json!({ "check_in_minutes": null })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let liga = "/tenants/argentina/tournaments/liga";
    let (status, body) = post_empty(&app, &format!("{liga}/publish"), &bia).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = post_empty(&app, &format!("{liga}/registration"), &bia).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");

    let list = mine(&app, &bia).await;
    assert_eq!(
        situations(&list),
        [
            ("quake-br/copa".to_owned(), "registered".to_owned()),
            ("argentina/liga".to_owned(), "registered".to_owned()),
        ]
    );
    assert_eq!(list[0]["tenant_name"], "Tenant quake-br");
    assert_eq!(list[0]["next_match"], Value::Null);
    assert_eq!(entry(&app, &bia, "copa").await["situation"], "registered");
    // Num torneio em que não está inscrita, a entrada não existe.
    let (status, body) = get(&app, "/tenants/quake-br/tournaments/outro/mine", &bia).await;
    assert_eq!(
        (status, body),
        (StatusCode::NOT_FOUND, err("participant.not_found"))
    );

    // O check-in abre: pendente até fazer.
    starts_in(&db, "copa", 10).await;
    assert_eq!(mine(&app, &bia).await[0]["situation"], "check_in_pending");
    for who in [&bia, &ana] {
        let (status, _) = post_empty(&app, &format!("{COPA}/check-in"), who).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }
    assert_eq!(mine(&app, &bia).await[0]["situation"], "checked_in");
    assert_eq!(mine(&app, &caio).await[0]["situation"], "check_in_pending");

    // Começou: Caio, sem check-in, ficou fora da chave.
    let (status, body) = post_empty(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let caio_entry = entry(&app, &caio, "copa").await;
    assert_eq!(caio_entry["situation"], "missed_check_in");
    assert_eq!(caio_entry["tournament"]["status"], "in_progress");
    assert_eq!(mine(&app, &bia).await[0]["situation"], "playing");
}

#[tokio::test]
async fn a_tdm_reserve_before_the_bracket() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let caio = member(&app, &db, &ana, "Caio").await;
    create_tournament(&app, &ana, crate::teams::tdm(2, json!({}))).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&ana, &bia]).await;
    assert_eq!(mine(&app, &bia).await[0]["situation"], "registered");

    // Com o time formado, quem se inscreve depois fica de reserva, também na
    // página da pessoa.
    let (status, body) = crate::teams::draw(&app, &ana, "random").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    register(&app, "copa", &[&caio]).await;
    assert_eq!(entry(&app, &caio, "copa").await["situation"], "reserve");
    assert_eq!(mine(&app, &bia).await[0]["situation"], "registered");
    let uri = format!("/tenants/quake-br/members/{}", caio.id);
    let (status, body) = get(&app, &uri, &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["tournaments"][0]["situation"], "reserve");
}

/// O lado (`"a"` ou `"b"`) de `who` na partida `key`.
async fn side_of(app: &Router, who: &User, key: &str) -> &'static str {
    let b = bracket(app, who).await;
    if side_player(&b, key, "a") == who.id {
        "a"
    } else {
        "b"
    }
}

#[tokio::test]
async fn next_match_and_placement() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p: Vec<User> = crate::matches::players(&app, &db, &ana, 4).await;
    start(
        &app,
        &ana,
        copa(single(1, 1)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;

    // P1 (seed 1) contra P4 na primeira rodada: enviar o mapa.
    let first = &entry(&app, &p[0], "copa").await;
    assert_eq!(first["situation"], "playing");
    let next = &first["next_match"];
    assert_eq!(next["key"], "wb-1-1");
    assert_eq!(next["state"], "ready");
    assert_eq!(next["single"], true);
    // O adversário é um time; no duelo, de um jogador, sem nome.
    assert_eq!(
        (
            &next["opponent"]["name"],
            &next["opponent"]["members"][0]["nickname"]
        ),
        (&Value::Null, &json!("P4"))
    );
    assert_eq!(next["task"], "report");

    // Enviou: espera o adversário, que tem de responder.
    let side = side_of(&app, &p[0], "wb-1-1").await;
    let (status, body) = report(&app, &p[0], "wb-1-1", 1, game(side, "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        mine(&app, &p[0]).await[0]["next_match"]["task"],
        "await_opponent"
    );
    assert_eq!(mine(&app, &p[3]).await[0]["next_match"]["task"], "respond");

    // Confirmado: P1 espera o vencedor da outra semifinal; P4 saiu em 3º–4º.
    let (status, body) = confirm(&app, &p[3], "wb-1-1", 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let next = &mine(&app, &p[0]).await[0]["next_match"];
    assert_eq!(next["key"], "wb-2-1");
    assert_eq!(next["state"], "pending");
    assert_eq!(next["opponent"], Value::Null);
    assert_eq!(
        next["opponent_from"],
        json!({ "outcome": "winner", "match_key": "wb-1-2" })
    );
    assert_eq!(next["task"], Value::Null);
    let out = &entry(&app, &p[3], "copa").await;
    assert_eq!(out["situation"], "eliminated");
    assert_eq!(out["placement"], json!({ "rank": 3, "tied": 2 }));
    assert_eq!(out["next_match"], Value::Null);

    // Final: P1 campeão, o torneio encerrado continua na lista. A lista lê a
    // colocação de todos os encerrados de uma vez, sem o veto; a rota de um
    // torneio só, da fase inteira: as duas dizem o mesmo (`entry`).
    let (a, b) = if side_of(&app, &p[1], "wb-1-2").await == "a" {
        (1, 0)
    } else {
        (0, 1)
    };
    play(&app, &p[1], &p[2], "wb-1-2", a, b).await;
    let (a, b) = if side_of(&app, &p[0], "wb-2-1").await == "a" {
        (1, 0)
    } else {
        (0, 1)
    };
    play(&app, &p[0], &p[1], "wb-2-1", a, b).await;
    let won = &entry(&app, &p[0], "copa").await;
    assert_eq!(won["tournament"]["status"], "finished");
    assert_eq!(won["situation"], "champion");
    assert_eq!(won["placement"], json!({ "rank": 1, "tied": 1 }));
    let second = &entry(&app, &p[1], "copa").await;
    assert_eq!(second["situation"], "eliminated");
    assert_eq!(second["placement"], json!({ "rank": 2, "tied": 1 }));
}
