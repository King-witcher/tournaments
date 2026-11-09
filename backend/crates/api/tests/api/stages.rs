//! Fases ligadas: a fase seguinte recebe quem jogou a anterior, pela
//! classificação final dela (corte, check-in com suplentes, desistentes
//! fora), começa pelo botão da organização e encerra o torneio só na última,
//! com a classificação composta; a correção na fase anterior que muda quem
//! entrou é recusada, e o início da fase se desfaz sem nada jogado.

use axum::{Router, http::StatusCode};
use serde_json::{Value, json};

use crate::common::*;
use crate::matches::{
    COPA, MAPS, bracket, confirm, contest, copa, detail, favorite_wins, find,
    finished::{finish_notices, placed, won},
    game, keys_in, players, report, set_game, set_result, sides, start, walkover,
};
use crate::realtime::{clear, notices, outbox};

fn stage_uri(n: u16) -> String {
    format!("{COPA}/stages/{n}")
}

/// Todos contra todos md1 e, depois, a disputa de posições md1 com `entry`
/// (`advance`, `check_in`).
fn linked(entry: Value) -> Value {
    let second = with(json!({ "format": "placement", "best_of": 1 }), entry);
    copa(json!({ "stages": [{ "format": "round_robin", "best_of": 1 }, second] }))
}

/// Joga todas as séries prontas da fase atual, o favorito (o de cima em `p`)
/// vencendo, até não sobrar nenhuma.
async fn play_out(app: &Router, p: &[User]) {
    loop {
        let b = bracket(app, &p[0]).await;
        let ready = keys_in(&b, "ready");
        if ready.is_empty() {
            return;
        }
        for key in ready {
            favorite_wins(app, p, &key, 1).await;
        }
    }
}

/// `who` vence (no lado dele) o mapa 1 da série md1 `key`, confirmado por
/// `other`.
async fn wins(app: &Router, who: &User, other: &User, key: &str) {
    let b = bracket(app, who).await;
    let (a, _) = sides(&b, find(&b, key));
    let side = if a == who.id { "a" } else { "b" };
    let (status, body) = report(app, who, key, 1, game(side, MAPS[0])).await;
    assert_eq!(status, StatusCode::OK, "{key}: {body}");
    let (status, body) = confirm(app, other, key, 1).await;
    assert_eq!(status, StatusCode::OK, "{key}: {body}");
}

async fn stage(app: &Router, who: &User, n: u16) -> Value {
    let (status, body) = get(app, &stage_uri(n), who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

async fn tournament(app: &Router, who: &User) -> Value {
    let (status, body) = get(app, COPA, who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

async fn mine(app: &Router, who: &User) -> Value {
    let (status, body) = get(app, "/me/tournaments", who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["registered"][0].clone()
}

/// O usuário do time `team` (duelo), pelos times da chave e os das fases
/// anteriores.
fn user_of(b: &Value, team: &Value) -> Value {
    let teams = b["entrants"]
        .as_array()
        .unwrap()
        .iter()
        .chain(b["other_teams"].as_array().unwrap());
    teams
        .clone()
        .find(|t| t["id"] == *team)
        .map_or(Value::Null, |t| t["members"][0]["user_id"].clone())
}

/// A classificação final `(usuário, posição)`, pela posição.
fn final_standings(b: &Value) -> Vec<(Value, Value)> {
    let mut list: Vec<(Value, Value)> = b["standings_final"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| (user_of(b, &s["team_id"]), s["rank"].clone()))
        .collect();
    list.sort_by_key(|(_, rank)| rank.as_i64());
    list
}

/// A série da fase 1 entre `x` e `y` e o lado de `x` nela.
fn series_of(b: &Value, x: &User, y: &User) -> (String, &'static str) {
    let pair = |m: &Value| sides(b, m);
    b["matches"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|m| {
            let key = m["key"].as_str()?.to_owned();
            match pair(m) {
                (a, c) if a == x.id && c == y.id => Some((key, "a")),
                (a, c) if a == y.id && c == x.id => Some((key, "b")),
                _ => None,
            }
        })
        .unwrap()
}

/// `(seed projetado, usuário)` dos candidatos.
fn projected(s: &Value) -> Vec<(Value, Value)> {
    s["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            (
                c["projected_seed"].clone(),
                c["team"]["members"][0]["user_id"].clone(),
            )
        })
        .collect()
}

async fn team_of(app: &Router, ana: &User, who: &User) -> i64 {
    team_ids(app, ana, "copa", &[who]).await[0]
}

fn check_in_uri(n: u16, team: i64) -> String {
    format!("{COPA}/stages/{n}/teams/{team}/check-in")
}

#[tokio::test]
async fn round_robin_then_placement_with_check_in_and_a_substitute() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 5).await;
    let all: Vec<&User> = p.iter().collect();
    let body = linked(json!({ "advance": 4, "check_in": true }));
    start(&app, &ana, body, &all).await;

    let t = tournament(&app, &ana).await;
    let status = |t: &Value| -> Vec<Value> {
        t["stages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["status"].clone())
            .collect()
    };
    assert_eq!(status(&t), [json!("in_progress"), json!("pending")]);
    assert_eq!(
        (&t["stages"][1]["number"], &t["stages"][1]["advance"]),
        (&json!(2), &json!(4))
    );
    // Antes do fim da fase 1: candidatos na ordem de agora, sem poder iniciar.
    let s = stage(&app, &p[0], 2).await;
    assert_eq!(s["start_blocker"], "stage.previous_not_finished");
    assert_eq!(s["can_start"], false);
    assert_eq!(s["candidates"].as_array().unwrap().len(), 5);
    let (status_code, body) =
        post_empty(&app, &format!("{}/check-in/open", stage_uri(2)), &ana).await;
    assert_eq!(
        (status_code, body),
        (StatusCode::CONFLICT, err("stage.previous_not_finished"))
    );
    // A fase 1 não tem rota de fase; a chave da 2 não existe antes do início.
    assert_eq!(
        get(&app, &stage_uri(1), &ana).await,
        (StatusCode::NOT_FOUND, err("stage.not_found"))
    );
    assert_eq!(
        get(&app, &format!("{COPA}/bracket?stage=2"), &ana).await,
        (StatusCode::NOT_FOUND, err("stage.not_started"))
    );

    play_out(&app, &p).await;
    // A fase 1 terminou: o torneio continua, e a fase 2 espera a organização.
    let t = tournament(&app, &ana).await;
    assert_eq!(t["status"], "in_progress");
    assert_eq!(status(&t), [json!("finished"), json!("pending")]);
    let (_, organizing) = get(&app, "/me/organizing", &ana).await;
    assert_eq!(organizing["stage_due"][0]["stage"], 2);
    // Antes de o check-in abrir, quem está acima do corte espera a fase 2; o 5º
    // é suplente.
    let me = mine(&app, &p[0]).await;
    assert_eq!(
        (&me["situation"], &me["stage"]),
        (&json!("awaiting_stage"), &json!(2))
    );
    assert_eq!(mine(&app, &p[4]).await["situation"], "substitute");
    let s = stage(&app, &ana, 2).await;
    assert_eq!(s["start_blocker"], "stage.check_in_not_open");

    clear(&db).await;
    let open = format!("{}/check-in/open", stage_uri(2));
    let (code, s) = post_empty(&app, &open, &ana).await;
    assert_eq!(code, StatusCode::OK, "{s}");
    assert_eq!(s["status"], "check_in");
    let opened: Vec<(String, Vec<i64>)> = notices(&outbox(&db).await)
        .into_iter()
        .filter(|(code, ..)| code == "stage.check_in_opened")
        .map(|(code, to, _)| (code, to))
        .collect();
    let mut everyone: Vec<i64> = p.iter().map(|u| u.id).collect();
    everyone.sort_unstable();
    assert_eq!(opened, [("stage.check_in_opened".to_owned(), everyone)]);
    assert_eq!(
        post_empty(&app, &open, &ana).await,
        (StatusCode::CONFLICT, err("stage.check_in_already_open"))
    );
    assert_eq!(
        mine(&app, &p[0]).await["situation"],
        "stage_check_in_pending"
    );

    // O 2º não confirma: o 5º sobe. O 3º é confirmado pela organização; o 2º
    // não confirma pelo 1º.
    let team = |i: usize| team_of(&app, &ana, &p[i]);
    // O 2º confirma e desfaz.
    let uri = check_in_uri(2, team(1).await);
    let (code, s) = post_empty(&app, &uri, &p[1]).await;
    assert_eq!(
        (code, &s["candidates"][1]["checked_in"]),
        (StatusCode::OK, &json!(true))
    );
    let (code, s) = delete(&app, &uri, &p[1]).await;
    assert_eq!(
        (code, &s["candidates"][1]["checked_in"]),
        (StatusCode::OK, &json!(false))
    );
    for i in [0, 3, 4] {
        let (code, body) = post_empty(&app, &check_in_uri(2, team(i).await), &p[i]).await;
        assert_eq!(code, StatusCode::OK, "{body}");
    }
    let (code, body) = post_empty(&app, &check_in_uri(2, team(2).await), &ana).await;
    assert_eq!(code, StatusCode::OK, "{body}");
    assert_eq!(
        post_empty(&app, &check_in_uri(2, team(0).await), &p[1]).await,
        (StatusCode::FORBIDDEN, err("stage.not_your_team"))
    );
    let s = stage(&app, &ana, 2).await;
    let id = |i: usize| json!(p[i].id);
    assert_eq!(
        projected(&s),
        [
            (json!(1), id(0)),
            (Value::Null, id(1)),
            (json!(2), id(2)),
            (json!(3), id(3)),
            (json!(4), id(4)),
        ]
    );
    assert_eq!(s["can_start"], true);
    assert_eq!(mine(&app, &p[4]).await["situation"], "awaiting_stage");

    clear(&db).await;
    let (code, s) = post_empty(&app, &format!("{}/start", stage_uri(2)), &ana).await;
    assert_eq!(code, StatusCode::OK, "{s}");
    assert_eq!(s["status"], "in_progress");
    let seeds: Vec<Value> = s["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["seed"].clone())
        .collect();
    assert_eq!(seeds, [json!(1), Value::Null, json!(2), json!(3), json!(4)]);
    // O check-in fechou com o início.
    assert_eq!(
        post_empty(&app, &check_in_uri(2, team(1).await), &p[1]).await,
        (StatusCode::CONFLICT, err("stage.check_in_not_open"))
    );
    // A chave da fase 2, com o prefixo na partida: 1 x 3 e 4 x 5.
    let b = bracket(&app, &p[0]).await;
    assert_eq!(b["stage"], 2);
    assert_eq!(keys_in(&b, "ready"), ["s2-pl-1-1", "s2-pl-1-2"]);
    assert_eq!(sides(&b, find(&b, "s2-pl-1-1")), (id(0), id(2)));
    assert_eq!(sides(&b, find(&b, "s2-pl-1-2")), (id(3), id(4)));
    let events = outbox(&db).await;
    let ready: Vec<String> = notices(&events)
        .into_iter()
        .filter(|(code, ..)| code == "match.ready")
        .map(|(_, _, params)| params["match"].as_str().unwrap().to_owned())
        .collect();
    assert!(ready.iter().all(|k| k.starts_with("s2-pl-1-")), "{ready:?}");
    assert!(
        events
            .iter()
            .any(|e| e["announce"]["code"] == "stage.started"
                && e["announce"]["params"]["stage"] == 2)
    );
    // O 2º, que não confirmou, já tem a posição: depois dos 4 que entraram.
    let me = mine(&app, &p[1]).await;
    assert_eq!(
        (&me["situation"], &me["placement"]["rank"]),
        (&json!("eliminated"), &json!(5))
    );
    let me = mine(&app, &p[4]).await;
    assert_eq!(
        (&me["situation"], &me["next_match"]["key"]),
        (&json!("playing"), &json!("s2-pl-1-2"))
    );
    // A fase 1 continua à mão.
    let (code, one) = get(&app, &format!("{COPA}/bracket?stage=1"), &p[0]).await;
    assert_eq!((code, &one["stage"]), (StatusCode::OK, &json!(1)));
    let d = detail(&app, &p[0], "s2-pl-1-1").await;
    assert_eq!(
        (&d["stage"], &d["summary"]["key"]),
        (&json!(2), &json!("s2-pl-1-1"))
    );
    assert_eq!(
        get(&app, &format!("{COPA}/matches/pl-1-1"), &p[0]).await,
        (StatusCode::NOT_FOUND, err("match.not_found"))
    );

    clear(&db).await;
    wins(&app, &p[2], &p[0], "s2-pl-1-1").await;
    wins(&app, &p[3], &p[4], "s2-pl-1-2").await;
    let b = bracket(&app, &p[0]).await;
    assert_eq!(b["status"], "finished");
    assert_eq!(
        final_standings(&b),
        [
            (id(2), json!(1)),
            (id(0), json!(2)),
            (id(3), json!(3)),
            (id(4), json!(4)),
            (id(1), json!(5)),
        ]
    );
    let finished = outbox(&db)
        .await
        .into_iter()
        .find(|e| e["announce"]["code"] == "tournament.finished")
        .unwrap();
    let podium: Vec<i64> = finished["announce"]["params"]["podium"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["rank"].as_i64().unwrap())
        .collect();
    assert_eq!(podium, [1, 2, 3]);
    assert_eq!(mine(&app, &p[2]).await["situation"], "champion");
    assert_eq!(mine(&app, &p[1]).await["placement"]["rank"], 5);
}

#[tokio::test]
async fn the_cut_leaves_out_who_withdrew_first() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    let all: Vec<&User> = p.iter().collect();
    start(&app, &ana, linked(json!({ "advance": 2 })), &all).await;
    play_out(&app, &p).await;
    // Sem check-in: o 1º e o 2º entrariam; o 2º desiste e o 3º sobe.
    assert_eq!(mine(&app, &p[2]).await["situation"], "substitute");
    let team = team_of(&app, &ana, &p[1]).await;
    let (code, body) = post_empty(&app, &format!("{COPA}/teams/{team}/withdrawal"), &ana).await;
    assert_eq!(code, StatusCode::NO_CONTENT, "{body}");
    assert_eq!(mine(&app, &p[2]).await["situation"], "awaiting_stage");
    assert_eq!(mine(&app, &p[1]).await["situation"], "withdrawn");
    let (code, s) = post_empty(&app, &format!("{}/start", stage_uri(2)), &ana).await;
    assert_eq!(code, StatusCode::OK, "{s}");
    let b = bracket(&app, &p[0]).await;
    let id = |i: usize| json!(p[i].id);
    assert_eq!(sides(&b, find(&b, "s2-pl-1-1")), (id(0), id(2)));
    // O 4º, cortado, fica à frente do 2º, que desistiu.
    assert_eq!(final_standings(&b), [(id(3), json!(3)), (id(1), json!(4))]);
    clear(&db).await;
    wins(&app, &p[0], &p[2], "s2-pl-1-1").await;
    let b = bracket(&app, &p[0]).await;
    assert_eq!(b["status"], "finished");
    assert_eq!(
        final_standings(&b),
        [
            (id(0), json!(1)),
            (id(2), json!(2)),
            (id(3), json!(3)),
            (id(1), json!(4))
        ]
    );
    // O 4º, cortado, fica sabendo; o 2º, que desistiu, não.
    assert_eq!(
        finish_notices(&db).await,
        [
            won(&p[0]),
            placed(&p[2], "P1", "2"),
            placed(&p[3], "P1", "3")
        ]
    );
    let me = mine(&app, &p[1]).await;
    assert_eq!(
        (&me["situation"], &me["placement"]["rank"]),
        (&json!("withdrawn"), &json!(4))
    );
}

#[tokio::test]
async fn a_correction_in_the_previous_stage_passes_only_without_changing_who_entered() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    let all: Vec<&User> = p.iter().collect();
    start(&app, &ana, linked(json!({})), &all).await;
    // Fase 2 com fase 1 por terminar: não começa.
    let start_uri = format!("{}/start", stage_uri(2));
    assert_eq!(
        post_empty(&app, &start_uri, &ana).await,
        (StatusCode::CONFLICT, err("stage.previous_not_finished"))
    );
    play_out(&app, &p).await;
    let (code, body) = post_empty(&app, &start_uri, &ana).await;
    assert_eq!(code, StatusCode::OK, "{body}");

    // O mapa de 1 x 2 corrigido com outros frags: a ordem não muda.
    let one = bracket(&app, &p[0]).await;
    assert_eq!(one["stage"], 2);
    let (_, b1) = get(&app, &format!("{COPA}/bracket?stage=1"), &ana).await;
    let (key, side) = series_of(&b1, &p[0], &p[1]);
    let other = if side == "a" { "b" } else { "a" };
    let (a_frags, b_frags) = if side == "a" { (20, 3) } else { (3, 20) };
    let frags = json!({ "map": MAPS[0], "a_frags": a_frags, "b_frags": b_frags });
    let (code, body) = set_game(&app, &ana, &key, 1, frags, None).await;
    assert_eq!(code, StatusCode::OK, "{body}");
    // O W.O. para o 2º muda a ordem da fase 2: recusado.
    let refused = (
        StatusCode::CONFLICT,
        json!({ "error": "stage.next_started", "params": { "stage": 2 } }),
    );
    assert_eq!(
        set_result(&app, &ana, &key, walkover(other), None).await,
        refused
    );
    // A contestação tardia da fase 1 fechou com a fase 2 (o 4º nunca enviou).
    let (late, _) = series_of(&b1, &p[3], &p[0]);
    let (code, body) = contest(&app, &p[3], &late, 1, "errado").await;
    assert_eq!(
        (code, &body["error"]),
        (StatusCode::CONFLICT, &json!("match.contest_closed"))
    );

    // Sem nada jogado na fase 2, o início se desfaz, e a correção passa.
    let (code, s) = delete(&app, &start_uri, &ana).await;
    assert_eq!(
        (code, &s["status"]),
        (StatusCode::OK, &json!("pending")),
        "{s}"
    );
    let (code, body) = set_result(&app, &ana, &key, walkover(other), None).await;
    assert_eq!(code, StatusCode::OK, "{body}");
    let (code, body) = post_empty(&app, &start_uri, &ana).await;
    assert_eq!(code, StatusCode::OK, "{body}");
    let b = bracket(&app, &p[0]).await;
    assert_eq!(
        sides(&b, find(&b, "s2-pl-1-1")),
        (json!(p[1].id), json!(p[0].id))
    );
    // Com um mapa enviado, não se desfaz mais; nem depois de ele ser anulado
    // para ser jogado de novo (o histórico da série fica).
    let (code, body) = report(&app, &p[1], "s2-pl-1-1", 1, game("a", MAPS[0])).await;
    assert_eq!(code, StatusCode::OK, "{body}");
    assert_eq!(
        delete(&app, &start_uri, &ana).await,
        (StatusCode::CONFLICT, err("stage.has_activity"))
    );
    let replay = format!("{COPA}/matches/s2-pl-1-1/games/1/replay");
    let (code, body) = post(&app, &replay, &ana, json!({})).await;
    assert_eq!(code, StatusCode::OK, "{body}");
    assert_eq!(detail(&app, &ana, "s2-pl-1-1").await["games"], json!([]));
    assert_eq!(
        delete(&app, &start_uri, &ana).await,
        (StatusCode::CONFLICT, err("stage.has_activity"))
    );
}

#[tokio::test]
async fn a_late_contest_reopens_the_stage_the_deadline_had_just_finished() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    let all: Vec<&User> = p.iter().collect();
    start(&app, &ana, linked(json!({})), &all).await;
    // Todas as séries menos 1 x 2; nela, o 1º envia e o prazo vence.
    let b = bracket(&app, &p[0]).await;
    let (last, side) = series_of(&b, &p[0], &p[1]);
    for key in keys_in(&b, "ready").into_iter().filter(|k| *k != last) {
        favorite_wins(&app, &p, &key, 1).await;
    }
    let (code, body) = report(&app, &p[0], &last, 1, game(side, MAPS[0])).await;
    assert_eq!(code, StatusCode::OK, "{body}");
    // O prazo vence: todos os mapas um dia antes, na mesma ordem (o envio
    // continua depois das outras séries, senão a contestação tardia fecharia
    // por elas).
    sqlx::query(
        "update match_games set reported_at = reported_at - interval '1 day',
                                confirm_by = confirm_by - interval '1 day',
                                confirmed_at = confirmed_at - interval '1 day'",
    )
    .execute(&db)
    .await
    .unwrap();
    // Na mesma requisição, a confirmação por tempo termina a fase 1 e a
    // contestação do 2º a reabre: a fase fica em andamento.
    let (code, body) = contest(&app, &p[1], &last, 1, "não foi isso").await;
    assert_eq!(code, StatusCode::OK, "{body}");
    let t = tournament(&app, &ana).await;
    assert_eq!(t["stages"][0]["status"], "in_progress");
    let history = crate::matches::kinds(&detail(&app, &ana, &last).await);
    for kind in ["auto_confirmed:1", "decided", "contested:1", "reopened"] {
        assert!(history.iter().any(|k| k == kind), "{kind}: {history:?}");
    }
    let (_, organizing) = get(&app, "/me/organizing", &ana).await;
    assert_eq!(organizing["stage_due"], json!([]));
    let finished: Option<i64> = sqlx::query_scalar(
        "select count(*) from tournament_stages where position = 1 and finished_at is not null",
    )
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(finished, Some(0));
}

#[tokio::test]
async fn stages_that_did_not_start_change_and_leave() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    // Eliminação só na última fase.
    let refused = post(
        &app,
        "/tenants/quake-br/tournaments",
        &ana,
        copa(json!({ "stages": [
            { "format": "elimination", "bracket": "single", "best_of": 1, "finals_best_of": 1 },
            { "format": "placement", "best_of": 1 }
        ] })),
    )
    .await;
    assert_eq!(
        refused,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({ "error": "stage.elimination_not_last", "params": { "stage": 1 } })
        )
    );

    let p = players(&app, &db, &ana, 4).await;
    let all: Vec<&User> = p.iter().collect();
    let three = |second: u8| {
        json!({ "stages": [
            { "format": "round_robin", "best_of": 1 },
            { "format": "placement", "best_of": second },
            { "format": "placement", "best_of": 1 }
        ] })
    };
    start(&app, &ana, copa(three(1)), &all).await;
    // Durante a fase 1: a fase 2 muda e a 3 sai; nada entra e a 1 não muda.
    let two = json!({ "stages": [
        { "format": "round_robin", "best_of": 1 },
        { "format": "placement", "best_of": 3 }
    ] });
    let (code, body) = patch(&app, COPA, &ana, two.clone()).await;
    assert_eq!(code, StatusCode::OK, "{body}");
    assert_eq!(
        (
            body["stages"][1]["best_of"].clone(),
            body["stages"].as_array().unwrap().len()
        ),
        (json!(3), 2)
    );
    let locked = (StatusCode::CONFLICT, err("tournament.stages_locked"));
    assert_eq!(patch(&app, COPA, &ana, three(3)).await, locked);
    let other_first = json!({ "stages": [{ "format": "round_robin", "best_of": 3 }, { "format": "placement", "best_of": 3 }] });
    assert_eq!(patch(&app, COPA, &ana, other_first).await, locked);

    play_out(&app, &p).await;
    assert_eq!(tournament(&app, &ana).await["status"], "in_progress");
    // Sem a fase 2, a 1, terminada, é a última: o torneio encerra pela tabela.
    let one = json!({ "stages": [{ "format": "round_robin", "best_of": 1 }] });
    clear(&db).await;
    let (code, body) = patch(&app, COPA, &ana, one).await;
    assert_eq!(code, StatusCode::OK, "{body}");
    assert_eq!(body["status"], "finished");
    // O pódio sai no Discord, como no fim pela chave.
    let finished = outbox(&db)
        .await
        .into_iter()
        .find(|e| e["announce"]["code"] == "tournament.finished")
        .expect("o pódio do encerramento");
    let podium = finished["announce"]["params"]["podium"].as_array().unwrap();
    assert_eq!(podium.len(), 3);
    assert_eq!(podium[0]["player"], "P1");
    // Quem jogou é avisado, como no fim pela chave.
    assert_eq!(
        finish_notices(&db).await,
        [
            won(&p[0]),
            placed(&p[1], "P1", "2"),
            placed(&p[2], "P1", "3"),
            placed(&p[3], "P1", "4")
        ]
    );
    let b = bracket(&app, &p[0]).await;
    let ranks: Vec<Value> = b["standings_final"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["rank"].clone())
        .collect();
    assert_eq!(ranks, [json!(1), json!(2), json!(3), json!(4)]);
    assert_eq!(mine(&app, &p[0]).await["situation"], "champion");
}

#[tokio::test]
async fn no_repeat_in_the_tournament_counts_like_the_stage() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    // Pela regra do jogador, cada fase recomeça: o orçamento é o da fase de
    // maior pior caso, não a soma. Um todos contra todos md1 com um a mais que
    // o pool de campeões cabe sozinho (cada um joga N − 1), mas não somado à
    // série da disputa de posições.
    let rules = |scope: &str| {
        json!({ "bans_per_series": 0, "mirror": true, "same_team": false,
                "no_repeat": { "who": "player", "scope": scope } })
    };
    let pool: i64 = sqlx::query_scalar("select count(*) from champions where active")
        .fetch_one(&db)
        .await
        .unwrap();
    assert!(pool < 32, "o round robin vai até 32");
    for scope in ["stage", "tournament"] {
        let body = copa(json!({
            "slug": format!("copa-{scope}"),
            "max_participants": pool + 1,
            "champion_rules": rules(scope),
            "stages": [
                { "format": "round_robin", "best_of": 1 },
                { "format": "placement", "best_of": 1 }
            ]
        }));
        let (code, created) = post(&app, "/tenants/quake-br/tournaments", &ana, body).await;
        assert_eq!(code, StatusCode::CREATED, "{scope}: {created}");
    }
}
