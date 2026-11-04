//! Testes de `veto_rooms`: criar e os limites (`create`), entrar num lugar e os
//! papéis (`seats`), o veto até o fim, no duelo e nos times (`flow`), a ordem das
//! escolhas, quem começa e o que fica escondido às cegas (`pick_order`), os
//! prazos, a validade e a limpeza (`time`), o socket por token (`socket`), quantas
//! idas ao banco custa cada clique e a leitura do espectador (`cost`) e as regras
//! do banco (`schema`).

mod cost;
mod create;
mod flow;
mod pick_order;
mod schema;
mod seats;
mod socket;
mod time;

use axum::{
    Router,
    http::{Method, StatusCode},
};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;

/// Os links de uma sala, como o de gestão os vê.
pub struct Links {
    pub manage: String,
    pub spectator: String,
    /// Os lugares na ordem: o lado A, depois o B.
    pub seats: Vec<String>,
}

pub async fn create_room(app: &Router, who: Option<&User>, body: Value) -> (StatusCode, Value) {
    call(app, Method::POST, "/veto-rooms", who, Some(body)).await
}

/// `team_size` por lado, melhor de `best_of`, com o resto no padrão.
pub fn room(team_size: i16, best_of: i16) -> Value {
    json!({ "team_size": team_size, "best_of": best_of })
}

/// Cria a sala e devolve os links.
pub async fn links_of(app: &Router, body: Value) -> Links {
    let (status, created) = create_room(app, None, body).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let manage = created["token"].as_str().unwrap().to_owned();
    let (status, detail) = read(app, &manage).await;
    assert_eq!(status, StatusCode::OK, "{detail}");
    let links = &detail["links"];
    Links {
        manage,
        spectator: links["spectator"].as_str().unwrap().to_owned(),
        seats: links["seats"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["token"].as_str().unwrap().to_owned())
            .collect(),
    }
}

pub async fn read(app: &Router, token: &str) -> (StatusCode, Value) {
    call(
        app,
        Method::GET,
        &format!("/veto-rooms/{token}"),
        None,
        None,
    )
    .await
}

/// A sala pelo link, que precisa existir.
pub async fn detail(app: &Router, token: &str) -> Value {
    let (status, body) = read(app, token).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

pub async fn act(
    app: &Router,
    token: &str,
    action: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let uri = format!("/veto-rooms/{token}/{action}");
    call(app, Method::POST, &uri, None, body).await
}

/// Entra no lugar `token` com o nome `name`, sem login.
pub async fn enter(app: &Router, token: &str, name: &str) -> Value {
    let (status, body) = act(app, token, "join", Some(json!({ "name": name }))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

pub async fn ready(app: &Router, token: &str) -> (StatusCode, Value) {
    act(app, token, "ready", None).await
}

pub async fn step(app: &Router, token: &str, choice: Value) -> (StatusCode, Value) {
    act(app, token, "steps", Some(choice)).await
}

/// Um passo que precisa dar certo.
pub async fn ok_step(app: &Router, token: &str, choice: Value) -> Value {
    let (status, body) = step(app, token, choice.clone()).await;
    assert_eq!(status, StatusCode::OK, "{choice}: {body}");
    body
}

/// Todos entram (`P1`, `P2`...) e os dois lados ficam prontos. Devolve a sala
/// como o primeiro lugar a vê.
pub async fn started(app: &Router, links: &Links) -> Value {
    for (i, seat) in links.seats.iter().enumerate() {
        enter(app, seat, &format!("P{}", i + 1)).await;
    }
    let half = links.seats.len() / 2;
    assert_eq!(ready(app, &links.seats[0]).await.0, StatusCode::OK);
    let (status, body) = ready(app, &links.seats[half]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["veto"]["status"], "in_progress");
    detail(app, &links.seats[0]).await
}

/// O primeiro lugar do lado da vez (nos dois lados, o A).
pub fn on_turn<'a>(links: &'a Links, room: &Value) -> &'a str {
    let half = links.seats.len() / 2;
    if room["veto"]["turn"]["sides"][0] == "a" {
        &links.seats[0]
    } else {
        &links.seats[half]
    }
}

/// Faz os passos de quem está na vez, sempre a primeira opção, até a vez ser
/// `until` ou o veto acabar. Devolve a sala como a resposta do último passo a
/// mostra, e quantos passos deu.
pub async fn run_steps(app: &Router, links: &Links, until: Option<&str>) -> (Value, usize) {
    let mut room = detail(app, &links.seats[0]).await;
    let mut steps = 0;
    while room["veto"]["status"] == "in_progress" && room["veto"]["turn"]["action"] != json!(until)
    {
        let token = on_turn(links, &room);
        let seen = detail(app, token).await;
        let option = &seen["veto"]["options"][0];
        let choice = match seen["veto"]["turn"]["target"].as_str().unwrap() {
            "map" => json!({ "map": option, "step": seen["veto"]["step"] }),
            _ => json!({ "champion": option, "step": seen["veto"]["step"] }),
        };
        room = ok_step(app, token, choice).await;
        steps += 1;
    }
    (room, steps)
}

/// Volta `seconds` segundos no relógio da sala: os eventos e o próximo sorteio,
/// em vez de esperar o tempo do passo.
pub async fn age(db: &PgPool, seconds: i32) {
    // O histórico não muda (regra do banco): o teste desliga a regra para
    // envelhecer as datas.
    for sql in [
        "alter table veto_room_events disable trigger veto_room_events_guard".to_owned(),
        format!(
            "update veto_room_events set created_at = created_at - interval '{seconds} seconds'"
        ),
        "alter table veto_room_events enable trigger veto_room_events_guard".to_owned(),
        format!("update veto_rooms set due_at = due_at - interval '{seconds} seconds'"),
    ] {
        sqlx::query(sqlx::AssertSqlSafe(sql))
            .execute(db)
            .await
            .unwrap();
    }
}
