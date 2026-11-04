//! A ordem das escolhas de campeão na série de um torneio: alternada, por jogador
//! nos times, e às cegas, em qualquer tamanho. O \`engine\` cobre as regras; aqui é
//! o caminho pela rota e, sobretudo, **o que cada um vê**: quem é do outro lado,
//! colega, espectador, organizador, antes e depois de a janela fechar, em tudo o
//! que sai (a série, a chave, "Seus torneios", os avisos do outbox).

use axum::{Router, http::StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;

use super::tdm::open_window;
use super::*;
use crate::realtime::{clear, outbox};
use crate::teams::{started, tdm};

const KEY: &str = "wb-1-1";

fn uri(action: &str) -> String {
    format!("{COPA}/matches/{KEY}/veto/{action}")
}

async fn ready(app: &Router, who: &User, key: &str) {
    let uri = format!("{COPA}/matches/{key}/veto/ready");
    let (status, body) = post_empty(app, &uri, who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

async fn step(app: &Router, who: &User, body: Value) -> (StatusCode, Value) {
    post(app, &uri("steps"), who, body).await
}

async fn ok(app: &Router, who: &User, body: Value) -> Value {
    let (status, response) = step(app, who, body.clone()).await;
    assert_eq!(status, StatusCode::OK, "{body}: {response}");
    response
}

/// O pick de \`who\` para a partida \`game\`.
async fn pick(app: &Router, who: &User, game: u8, champion: &str) -> Value {
    ok(app, who, json!({ "champion": champion, "game": game })).await
}

fn rules(bans: u8, mirror: bool) -> Value {
    json!({ "bans_per_series": bans, "mirror": mirror, "same_team": false, "no_repeat": null })
}

/// Com \`changes\`, o veto de \`pick_order\`, sem mapas, e a série de \`best_of\`.
fn setup(pick_order: &str, bans: u8, best_of: u8) -> Value {
    with(
        json!({
            "veto": { "map_mode": "off", "champion_phase": true, "pick_order": pick_order },
            "champion_rules": rules(bans, true),
        }),
        single(best_of, best_of),
    )
}

/// 2v2 iniciado, os dois lados prontos: A = P1 e P2; B = P3 e P4. O espectador é
/// \`bia\`, que é da comunidade e não joga; \`ana\` organiza.
async fn two_v_two(db: &PgPool, changes: Value) -> (Router, User, User, Vec<User>) {
    let (app, ana, bia) = quake_br(db).await;
    let p = players(&app, db, &ana, 4).await;
    let body = tdm(2, changes);
    started(&app, &ana, body, 2, &[&p[0], &p[1], &p[2], &p[3]]).await;
    for who in &p {
        ready(&app, who, KEY).await;
    }
    (app, ana, bia, p)
}

/// Duelo iniciado, os dois prontos: A = P1, B = P2.
async fn duel_started(db: &PgPool, changes: Value) -> (Router, User, User, Vec<User>) {
    let (app, ana, bia) = quake_br(db).await;
    let p = players(&app, db, &ana, 2).await;
    start(&app, &ana, copa(changes), &[&p[0], &p[1]]).await;
    ready(&app, &p[0], KEY).await;
    ready(&app, &p[1], KEY).await;
    (app, ana, bia, p)
}

/// A resposta inteira, sem o que lista todos os campeões de propósito (a grade,
/// as opções e o motivo de cada um, conferidos à parte): o que sobra não pode
/// citar um campeão escolhido.
fn without_the_grid(body: &Value) -> String {
    let mut body = body.clone();
    if let Some(veto) = body.get_mut("veto").and_then(Value::as_object_mut) {
        veto.remove("champions");
        veto.remove("picks");
        veto.remove("options");
    }
    body.to_string()
}

fn assert_not_in(body: &Value, slug: &str, who: &str) {
    let text = without_the_grid(body);
    assert!(!text.contains(slug), "{who} viu {slug}: {text}");
}

fn assert_in(body: &Value, slug: &str, who: &str) {
    let text = without_the_grid(body);
    assert!(text.contains(slug), "{who} não viu {slug}: {text}");
}

fn turn_of(body: &Value) -> (String, Value) {
    let turn = &body["veto"]["turn"];
    (
        turn["sides"][0].as_str().unwrap().to_owned(),
        turn["pick_for"].clone(),
    )
}

/// O id do evento do pick: o histórico não pode dizer que ele existiu.
fn picks_in_history(body: &Value) -> usize {
    body["veto"]["history"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "pick")
        .count()
}

// A ordem alternada.

#[tokio::test]
async fn the_alternating_order_goes_player_by_player_and_each_pick_is_a_step() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _, _, p) = two_v_two(&db, setup("alternating", 0, 3)).await;
    let d = detail(&app, &p[0], KEY).await;
    assert_eq!(d["veto"]["pick_order"], "alternating");
    assert_eq!(d["veto"]["team_size"], 2);
    assert_eq!(d["veto"]["team_pick_seconds"], Value::Null);
    assert_eq!(d["veto"]["turn"]["action"], "team_pick");
    assert_eq!(turn_of(&d), ("a".to_owned(), json!(1)));
    // Os dois de A podem; os de B, não; cada um vê a tarefa que é dele.
    for who in [&p[0], &p[1]] {
        let seen = detail(&app, who, KEY).await;
        assert_eq!(seen["summary"]["task"], "veto");
        assert_eq!(seen["veto"]["picks"].as_array().unwrap().len(), 1);
        assert_eq!(seen["veto"]["picks"][0]["game"], 1);
    }
    let seen = detail(&app, &p[2], KEY).await;
    assert_eq!(seen["summary"]["task"], "await_opponent");
    assert_eq!(seen["veto"]["picks"], json!([]));
    assert_eq!(
        step(&app, &p[2], json!({ "champion": "anarki" })).await,
        (StatusCode::CONFLICT, err("veto.not_your_turn"))
    );

    // A partida é a que a vez diz: sem dizer, vale; outra é uma tela que ficou
    // para trás (a vez muda sem evento quando muda o elenco).
    assert_eq!(
        step(&app, &p[1], json!({ "champion": "anarki", "game": 3 })).await,
        (StatusCode::CONFLICT, err("veto.step_changed"))
    );
    let shown = d["veto"]["step"].clone();
    let after = ok(
        &app,
        &p[1],
        json!({ "champion": "anarki", "game": 1, "step": shown }),
    )
    .await;
    assert_eq!(
        after["veto"]["games"][0]["team_picks"][0]["user_id"],
        p[1].id
    );
    assert_eq!(after["veto"]["games"][2]["team_picks"], json!([]));
    // O passo muda a vez: a tela velha é recusada, e o prazo recomeça.
    assert_ne!(after["veto"]["step"], shown);
    assert_eq!(
        step(&app, &p[0], json!({ "champion": "athena", "step": shown })).await,
        (StatusCode::CONFLICT, err("veto.step_changed"))
    );
    assert_ne!(after["veto"]["deadline"], d["veto"]["deadline"]);
    assert_eq!(turn_of(&after), ("b".to_owned(), json!(1)));

    // B, B; depois P1, o que falta de A; quem já escolheu não é mais com ele.
    let after = ok(&app, &p[2], json!({ "champion": "anarki" })).await;
    assert_eq!(turn_of(&after), ("b".to_owned(), json!(1)));
    let after = ok(&app, &p[3], json!({ "champion": "clutch" })).await;
    assert_eq!(turn_of(&after), ("a".to_owned(), json!(1)));
    let done = detail(&app, &p[1], KEY).await;
    assert_eq!(done["summary"]["task"], "await_opponent");
    assert_eq!(done["veto"]["picks"], json!([]));
    assert_eq!(
        step(&app, &p[1], json!({ "champion": "clutch" })).await,
        (StatusCode::CONFLICT, err("veto.already_picked"))
    );
    let after = ok(&app, &p[0], json!({ "champion": "athena" })).await;
    // Na partida 2, quem abre é o outro.
    assert_eq!(turn_of(&after), ("b".to_owned(), json!(2)));
    // Tudo à vista de todos, o tempo todo.
    for who in [&p[0], &p[2]] {
        let seen = detail(&app, who, KEY).await;
        assert_eq!(
            seen["veto"]["games"][0]["team_picks"]
                .as_array()
                .unwrap()
                .len(),
            4
        );
        assert_eq!(seen["veto"]["games"][0]["hidden_picks"], json!([]));
    }
}

#[tokio::test]
async fn the_step_deadline_draws_only_for_the_first_of_the_side() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _, p) = two_v_two(&db, setup("alternating", 0, 1)).await;
    age_veto(&db, 33).await;
    let d = detail(&app, &ana, KEY).await;
    // 30 s por passo, com 2 s de tolerância: venceu um passo só, o de P1.
    let picks = d["veto"]["games"][0]["team_picks"].as_array().unwrap();
    assert_eq!(picks.len(), 1, "{d}");
    assert_eq!(picks[0]["user_id"], p[0].id);
    assert_eq!(picks[0]["timeout"], true);
    assert_eq!(turn_of(&d), ("b".to_owned(), json!(1)));
    let seconds = d["veto"]["team_pick_seconds"].clone();
    assert_eq!(seconds, Value::Null);
    // O tempo todo: cada um sorteado, um por passo, até o fim.
    age_veto(&db, 3600).await;
    let d = detail(&app, &ana, KEY).await;
    assert_eq!(d["veto"]["status"], "done", "{d}");
    assert_eq!(
        d["veto"]["games"][0]["team_picks"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
}

#[tokio::test]
async fn the_duel_is_as_it_always_was() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _, _, p) = duel_started(&db, setup("alternating", 0, 1)).await;
    let d = detail(&app, &p[0], KEY).await;
    assert_eq!(d["veto"]["turn"]["action"], "pick");
    assert_eq!(d["veto"]["team_size"], 1);
    assert_eq!(turn_of(&d), ("a".to_owned(), json!(1)));
    let after = ok(&app, &p[0], json!({ "champion": "anarki" })).await;
    assert_eq!(after["veto"]["games"][0]["pick_a"]["slug"], "anarki");
    assert_eq!(after["veto"]["games"][0]["team_picks"], json!([]));
    assert_eq!(turn_of(&after), ("b".to_owned(), json!(1)));
}

#[tokio::test]
async fn a_tournament_that_had_started_keeps_the_open_window() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _, _, p) = two_v_two(&db, setup("alternating", 0, 1)).await;
    open_window(&db).await;
    let d = detail(&app, &p[0], KEY).await;
    assert_eq!(d["veto"]["pick_order"], "open");
    assert_eq!(d["veto"]["turn"]["sides"], json!(["a", "b"]));
    assert_eq!(d["veto"]["team_pick_seconds"], 60);
    pick(&app, &p[0], 1, "anarki").await;
    // Aberta: quem é do outro lado vê.
    let seen = detail(&app, &p[2], KEY).await;
    assert_in(&seen, "anarki", "o adversário");
    assert_eq!(seen["veto"]["games"][0]["hidden_picks"], json!([]));
}

// As regras de configurar.

#[tokio::test]
async fn blind_needs_a_mirror_when_creating_and_when_editing() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let blind_no_mirror = copa(with(
        setup("blind", 1, 3),
        json!({ "champion_rules": rules(1, false) }),
    ));
    let tournaments = "/tenants/quake-br/tournaments";
    assert_eq!(
        post(&app, tournaments, &ana, blind_no_mirror).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("tournament.blind_needs_mirror")
        )
    );
    let created = create_tournament(&app, &ana, copa(setup("blind", 1, 3))).await;
    assert_eq!(created["veto"]["pick_order"], "blind");
    // Quem ligou às cegas e depois desliga o espelho.
    assert_eq!(
        patch(
            &app,
            COPA,
            &ana,
            json!({ "champion_rules": rules(1, false) })
        )
        .await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("tournament.blind_needs_mirror")
        )
    );
    // Os dois de uma vez passam, para o alternado; o caminho contrário, não.
    let (status, body) = patch(
        &app,
        COPA,
        &ana,
        json!({
            "champion_rules": rules(1, false),
            "veto": { "pick_order": "alternating" }
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        patch(
            &app,
            COPA,
            &ana,
            json!({ "veto": { "pick_order": "blind" } })
        )
        .await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("tournament.blind_needs_mirror")
        )
    );
}

#[tokio::test]
async fn the_open_window_is_not_chosen_anymore() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let tournaments = "/tenants/quake-br/tournaments";
    assert_eq!(
        post(&app, tournaments, &ana, copa(setup("open", 1, 3))).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("tournament.pick_order_invalid")
        )
    );
    create_tournament(&app, &ana, copa(setup("alternating", 1, 3))).await;
    assert_eq!(
        patch(
            &app,
            COPA,
            &ana,
            json!({ "veto": { "pick_order": "open" } })
        )
        .await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("tournament.pick_order_invalid")
        )
    );
}

#[tokio::test]
async fn undoing_the_start_takes_the_open_window_away() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _, _) = two_v_two(&db, setup("alternating", 0, 1)).await;
    open_window(&db).await;
    // Mandar de novo o que já está não é mudança, nem depois de começar.
    let (status, body) = patch(
        &app,
        COPA,
        &ana,
        json!({ "veto": { "pick_order": "open" } }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = delete(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["veto"]["pick_order"], "alternating");
}

#[tokio::test]
async fn the_budget_is_the_same_in_every_order() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let tournaments = "/tenants/quake-br/tournaments";
    // A mesma conta: 16 vagas em times de 2, no torneio inteiro, passa do pool.
    let over = |order: &str| {
        tdm(
            2,
            with(
                setup(order, 1, 3),
                json!({ "champion_rules": {
                    "bans_per_series": 1, "mirror": true, "same_team": false,
                    "no_repeat": { "who": "team", "scope": "tournament" }
                } }),
            ),
        )
    };
    let refused = post(&app, tournaments, &ana, over("alternating")).await;
    assert_eq!(refused.0, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(refused.1["error"], "champions.pool_exhausted");
    assert_eq!(post(&app, tournaments, &ana, over("blind")).await, refused);
}

// Às cegas: o que cada um vê.

#[tokio::test]
async fn blind_hides_the_picks_from_everyone_outside_the_side_until_it_closes() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia, p) = two_v_two(&db, setup("blind", 0, 1)).await;
    clear(&db).await;
    let d = detail(&app, &p[0], KEY).await;
    assert_eq!(d["veto"]["pick_order"], "blind");
    assert_eq!(d["veto"]["team_pick_seconds"], 60);
    assert_eq!(d["veto"]["turn"]["sides"], json!(["a", "b"]));
    assert_eq!(d["veto"]["turn"]["pick_for"], Value::Null);
    let window = d["veto"]["step"].clone();

    // P1 escolhe. A resposta dele e o colega veem; ninguém mais.
    let after = pick(&app, &p[0], 1, "anarki").await;
    assert_in(&after, "anarki", "quem escolheu");
    assert_eq!(after["veto"]["step"], window);
    let mate = detail(&app, &p[1], KEY).await;
    assert_in(&mate, "anarki", "o colega");
    assert_eq!(
        mate["veto"]["games"][0]["team_picks"][0]["user_id"],
        p[0].id
    );
    assert_eq!(picks_in_history(&mate), 1);
    let unavailable = &mate["veto"]["picks"][0]["unavailable"];
    assert_eq!(unavailable[0]["slug"], "anarki");
    assert_eq!(unavailable[0]["reason"], "teammate");

    let outsiders: [(&User, &str); 4] = [
        (&p[2], "o adversário"),
        (&p[3], "o outro adversário"),
        (&bia, "o espectador"),
        (&ana, "o organizador"),
    ];
    for (who, label) in outsiders {
        let seen = detail(&app, who, KEY).await;
        assert_not_in(&seen, "anarki", label);
        assert_eq!(seen["veto"]["games"][0]["team_picks"], json!([]), "{label}");
        assert_eq!(picks_in_history(&seen), 0, "{label}");
        assert_eq!(seen["veto"]["games"][0]["pick_a"], Value::Null, "{label}");
        // O progresso aparece; o que foi escolhido, não.
        let hidden = seen["veto"]["games"][0]["hidden_picks"].as_array().unwrap();
        assert_eq!(hidden.len(), 1, "{label}");
        assert_eq!(
            hidden[0],
            json!({
                "side": "a",
                "user_id": p[0].id,
                "nickname": hidden[0]["nickname"],
            })
        );
        assert_eq!(
            seen["veto"]["missing"].as_array().unwrap().len(),
            3,
            "{label}"
        );
        // A vez, o prazo e o número da vez não contam nada a mais.
        assert_eq!(seen["veto"]["step"], window, "{label}");
        assert_eq!(seen["veto"]["deadline"], d["veto"]["deadline"], "{label}");
    }
    // Os dois lados podem pegar o mesmo: o adversário tem o campeão nas opções, e
    // nenhum motivo o cita.
    let slot = &detail(&app, &p[2], KEY).await["veto"]["picks"][0].clone();
    assert!(
        slot["options"]
            .as_array()
            .unwrap()
            .contains(&json!("anarki"))
    );
    assert_eq!(slot["unavailable"], json!([]));
    assert_eq!(slot["gave_way"], Value::Null);

    // Nem a chave, nem "Seus torneios", nem os avisos, nem a recusa.
    let bracket = get(&app, &format!("{COPA}/bracket"), &bia).await.1;
    assert!(!bracket.to_string().contains("anarki"));
    let mine = get(&app, "/me/tournaments", &p[2]).await.1;
    assert!(!mine.to_string().contains("anarki"));
    let events = serde_json::to_string(&outbox(&db).await).unwrap();
    assert!(!events.contains("anarki"), "{events}");

    // O adversário escolhe o mesmo: passa, e os dois lados continuam sem se ver.
    let theirs = pick(&app, &p[2], 1, "anarki").await;
    assert_eq!(
        theirs["veto"]["games"][0]["team_picks"][0]["user_id"],
        p[2].id
    );
    assert_eq!(
        theirs["veto"]["games"][0]["team_picks"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let ours = detail(&app, &p[1], KEY).await;
    assert_eq!(
        ours["veto"]["games"][0]["team_picks"][0]["user_id"],
        p[0].id
    );
    assert_eq!(
        ours["veto"]["games"][0]["team_picks"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    // Fecha: agora todos veem tudo, no resumo e no histórico.
    pick(&app, &p[1], 1, "athena").await;
    pick(&app, &p[3], 1, "clutch").await;
    for (who, label) in outsiders {
        let seen = detail(&app, who, KEY).await;
        assert_eq!(seen["veto"]["status"], "done", "{label}");
        assert_eq!(
            seen["veto"]["games"][0]["team_picks"]
                .as_array()
                .unwrap()
                .len(),
            4
        );
        assert_eq!(seen["veto"]["games"][0]["hidden_picks"], json!([]));
        assert_eq!(picks_in_history(&seen), 4, "{label}");
        assert_in(&seen, "anarki", label);
    }
}

#[tokio::test]
async fn a_blind_window_closed_by_the_deadline_opens_what_was_drawn_too() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia, p) = two_v_two(&db, setup("blind", 0, 1)).await;
    pick(&app, &p[0], 1, "anarki").await;
    // Antes do prazo, escondido.
    assert_not_in(&detail(&app, &bia, KEY).await, "anarki", "o espectador");
    age_veto(&db, 63).await;
    let d = detail(&app, &ana, KEY).await;
    assert_eq!(d["veto"]["status"], "done", "{d}");
    let picks = d["veto"]["games"][0]["team_picks"].as_array().unwrap();
    assert_eq!(picks.len(), 4);
    assert_eq!(picks.iter().filter(|x| x["timeout"] == true).count(), 3);
    assert_in(&d, "anarki", "o organizador");
}

#[tokio::test]
async fn the_blind_duel_has_the_window_and_hides_the_other_side() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia, p) = duel_started(&db, setup("blind", 1, 3)).await;
    // Os bans continuam abertos e alternados, antes.
    let d = detail(&app, &p[0], KEY).await;
    assert_eq!(d["veto"]["turn"]["action"], "ban");
    let mut banned = Vec::new();
    for _ in 0..2 {
        let turn = detail(&app, &p[0], KEY).await;
        let side = turn["veto"]["turn"]["sides"][0]
            .as_str()
            .unwrap()
            .to_owned();
        let who = if side == "a" { &p[0] } else { &p[1] };
        let seen = detail(&app, who, KEY).await;
        let option = seen["veto"]["options"][0].clone();
        banned.push(option.as_str().unwrap().to_owned());
        ok(&app, who, json!({ "champion": option })).await;
    }
    let seen = detail(&app, &p[1], KEY).await;
    assert_eq!(seen["veto"]["bans"].as_array().unwrap().len(), 2);
    assert_eq!(seen["veto"]["turn"]["action"], "team_pick");
    assert_eq!(seen["veto"]["turn"]["sides"], json!(["a", "b"]));
    assert_eq!(seen["veto"]["picks"].as_array().unwrap().len(), 3);
    let (a1, b1) = ("clutch", "doom-slayer");

    pick(&app, &p[0], 2, a1).await;
    for (who, label) in [
        (&p[1], "o adversário"),
        (&bia, "o espectador"),
        (&ana, "o organizador"),
    ] {
        let seen = detail(&app, who, KEY).await;
        assert_not_in(&seen, a1, label);
        assert_eq!(seen["veto"]["games"][1]["pick_a"], Value::Null, "{label}");
        assert_eq!(
            seen["veto"]["games"][1]["hidden_picks"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }
    // Quem começa a janela no duelo tem o aviso, como nos times; sem o campeão.
    let events = serde_json::to_string(&outbox(&db).await).unwrap();
    assert!(events.contains("match.team_pick_open"), "{events}");
    assert!(!events.contains(a1), "{events}");

    // O resto, e abre.
    pick(&app, &p[1], 2, a1).await;
    for (who, game, slug) in [
        (&p[0], 1, b1),
        (&p[0], 3, "galena"),
        (&p[1], 1, b1),
        (&p[1], 3, "galena"),
    ] {
        pick(&app, who, game, slug).await;
    }
    let seen = detail(&app, &bia, KEY).await;
    assert_eq!(seen["veto"]["status"], "done");
    assert_eq!(
        seen["veto"]["games"][1]["team_picks"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_in(&seen, a1, "o espectador");
}

#[tokio::test]
async fn a_blind_pick_is_not_undone_by_anyone() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _, p) = two_v_two(&db, setup("blind", 1, 1)).await;
    let mut d = detail(&app, &p[0], KEY).await;
    for _ in 0..2 {
        let side = d["veto"]["turn"]["sides"][0].as_str().unwrap().to_owned();
        let who = if side == "a" { &p[0] } else { &p[2] };
        let option = detail(&app, who, KEY).await["veto"]["options"][0].clone();
        d = ok(&app, who, json!({ "champion": option })).await;
    }
    assert_eq!(d["veto"]["turn"]["action"], "team_pick");
    // Sem nenhum pick, o último ban ainda se desfaz: a janela não mostrou nada.
    assert_eq!(d["veto"]["can_undo"], true);
    let (status, undone) = post_empty(&app, &uri("undo"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{undone}");
    assert_eq!(undone["veto"]["turn"]["action"], "ban");
    let side = undone["veto"]["turn"]["sides"][0]
        .as_str()
        .unwrap()
        .to_owned();
    let who = if side == "a" { &p[0] } else { &p[2] };
    let option = detail(&app, who, KEY).await["veto"]["options"][0].clone();
    ok(&app, who, json!({ "champion": option })).await;

    // Com um pick, ninguém desfaz, nem pede: nem aberta, nem depois de fechar.
    pick(&app, &p[0], 1, "clutch").await;
    let refused = (StatusCode::CONFLICT, err("veto.nothing_to_undo"));
    assert_eq!(detail(&app, &ana, KEY).await["veto"]["can_undo"], false);
    assert_eq!(post_empty(&app, &uri("undo"), &ana).await, refused);
    assert_eq!(post_empty(&app, &uri("undo-request"), &p[2]).await, refused);
    pick(&app, &p[1], 1, "doom-slayer").await;
    pick(&app, &p[2], 1, "galena").await;
    let d = pick(&app, &p[3], 1, "keel").await;
    assert_eq!(d["veto"]["status"], "done");
    assert_eq!(d["veto"]["can_undo"], false);
    assert_eq!(post_empty(&app, &uri("undo"), &ana).await, refused);
}

#[tokio::test]
async fn the_reservation_between_series_does_not_tell_the_other_side() {
    // Todos contra todos às cegas, sem repetir no torneio: a mesma pessoa em duas
    // séries ao mesmo tempo. O que P1 escolheu numa fica reservado para ele na
    // outra, sem que ninguém mais saiba.
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 3).await;
    let body = copa(with(
        table("round_robin", json!({ "best_of": 1 })),
        json!({
            "veto": { "map_mode": "off", "champion_phase": true, "pick_order": "blind" },
            "champion_rules": {
                "bans_per_series": 0, "mirror": true, "same_team": false,
                "no_repeat": { "who": "player", "scope": "tournament" }
            },
        }),
    ));
    start(&app, &ana, body, &[&p[0], &p[1], &p[2]]).await;
    let b = bracket(&app, &p[0]).await;
    // As duas séries de P1.
    let of_p1: Vec<String> = b["matches"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["state"] == "ready")
        .filter(|m| {
            let (x, y) = sides(&b, m);
            x == p[0].id || y == p[0].id
        })
        .map(|m| m["key"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(of_p1.len(), 2, "{b}");
    let opponent_in = |key: &str| {
        let m = find(&b, key);
        let (x, y) = sides(&b, m);
        let other = if x == p[0].id { y } else { x };
        p.iter().find(|u| other == u.id).unwrap()
    };
    for key in &of_p1 {
        ready(&app, &p[0], key).await;
        ready(&app, opponent_in(key), key).await;
    }
    // P1 escolhe na primeira.
    let (first, second) = (&of_p1[0], &of_p1[1]);
    let steps = format!("{COPA}/matches/{first}/veto/steps");
    let (status, body) = post(
        &app,
        &steps,
        &p[0],
        json!({ "champion": "anarki", "game": 1 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // Na segunda, é só dele a reserva.
    let mine = detail(&app, &p[0], second).await;
    let reasons = &mine["veto"]["picks"][0]["unavailable"];
    assert_eq!(reasons[0], json!({ "slug": "anarki", "reason": "repeat" }));
    let rival = opponent_in(second);
    let theirs = detail(&app, rival, second).await;
    assert_eq!(theirs["veto"]["picks"][0]["unavailable"], json!([]));
    assert!(
        theirs["veto"]["picks"][0]["options"]
            .as_array()
            .unwrap()
            .contains(&json!("anarki"))
    );
    assert_not_in(&theirs, "anarki", "o adversário da outra série");
    // Quem está na primeira do outro lado, e quem só olha, também não.
    let across = opponent_in(first);
    assert_not_in(&detail(&app, across, first).await, "anarki", "o adversário");
    assert_not_in(&detail(&app, &bia, first).await, "anarki", "o espectador");
    assert_not_in(&detail(&app, &bia, second).await, "anarki", "o espectador");
}

#[tokio::test]
async fn a_game_already_sent_stays_in_view_when_the_window_reopens() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    let body = tdm(2, setup("blind", 0, 3));
    let teams = started(&app, &ana, body, 2, &[&p[0], &p[1], &p[2], &p[3]]).await;
    for who in &p {
        ready(&app, who, KEY).await;
    }
    let slugs = [
        "anarki",
        "athena",
        "clutch",
        "doom-slayer",
        "galena",
        "keel",
    ];
    for (who, offset) in [(&p[0], 0), (&p[1], 3)] {
        for game in 1u8..=3 {
            pick(&app, who, game, slugs[offset + usize::from(game) - 1]).await;
        }
    }
    for (who, offset) in [(&p[2], 0), (&p[3], 3)] {
        for game in 1u8..=3 {
            pick(&app, who, game, slugs[offset + usize::from(game) - 1]).await;
        }
    }
    assert_eq!(detail(&app, &bia, KEY).await["veto"]["status"], "done");
    // A partida 1 é enviada; a partida 2 começa, e um substituto entra.
    let (status, body) = report(&app, &p[0], KEY, 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let substitution = format!("{COPA}/teams/{}/substitution", teams[0]);
    let (status, body) = post(
        &app,
        &substitution,
        &ana,
        json!({ "out": p[1].id, "in": bia.id }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // A janela reabriu: o que ainda vale para as partidas abertas fica escondido
    // de quem não é do lado; a partida enviada continua à vista.
    let seen = detail(&app, &p[2], KEY).await;
    assert_eq!(seen["veto"]["turn"]["action"], "team_pick", "{seen}");
    let games = seen["veto"]["games"].as_array().unwrap();
    assert_eq!(games[0]["team_picks"].as_array().unwrap().len(), 4);
    assert_eq!(games[0]["hidden_picks"], json!([]));
    // O próprio lado segue; o de A, só quem já escolheu, sem o quê.
    assert_eq!(games[1]["hidden_picks"].as_array().unwrap().len(), 1);
    let own: Vec<&Value> = games[1]["team_picks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| &t["side"])
        .collect();
    assert_eq!(own, [&json!("b"), &json!("b")]);
}

/// Volta \`seconds\` segundos no relógio de todos os eventos de veto.
async fn age_veto(db: &PgPool, seconds: i32) {
    for sql in [
        "alter table match_veto_events disable trigger match_veto_events_guard".to_owned(),
        format!(
            "update match_veto_events set created_at = created_at - interval '{seconds} seconds'"
        ),
        "alter table match_veto_events enable trigger match_veto_events_guard".to_owned(),
    ] {
        sqlx::query(sqlx::AssertSqlSafe(sql))
            .execute(db)
            .await
            .unwrap();
    }
}
