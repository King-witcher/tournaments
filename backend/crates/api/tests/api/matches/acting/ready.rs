//! O "Pronto": só o capitão, ou todos do elenco de agora, com o elenco mudando no meio.

use api::realtime::veto_clock;

use super::*;
use crate::schema::count;

// Pronto.

#[tokio::test]
async fn with_everything_on_the_captain_only_the_captain_marks_ready() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cast(&db, all_captain(), draft("alternating", 1, 1)).await;
    let [cap, mate, other_cap, other_mate] = [&c.p[0], &c.p[1], &c.p[2], &c.p[3]];
    let seen = detail(&c.app, mate, KEY).await;
    assert_eq!(seen["veto"]["ready_by"], "captain");
    assert_eq!(seen["veto"]["can_act"]["ready"], false);
    assert_eq!(seen["summary"]["task"], "await_captain");
    assert_eq!(
        users(&seen["veto"]["ready_waiting"]),
        [cap.id, other_cap.id]
    );
    let not_captain = (StatusCode::FORBIDDEN, err(NOT_CAPTAIN));
    assert_eq!(ready(&c, mate).await, not_captain);
    assert_eq!(ready(&c, other_mate).await, not_captain);
    assert_eq!(unready(&c, mate).await, not_captain);
    let not_player = (StatusCode::FORBIDDEN, err(NOT_PLAYER));
    assert_eq!(ready(&c, &c.ana).await, not_player);
    assert_eq!(ready(&c, &c.bia).await, not_player);

    clear(&db).await;
    let body = ok_ready(&c, cap).await;
    assert_eq!(body["veto"]["ready_a"], true);
    assert_eq!(body["summary"]["task"], "await_opponent");
    // O adversário recebe o aviso, e só o capitão dele pode responder.
    let sent = notices(&outbox(&db).await);
    let asked: Vec<&Vec<i64>> = sent
        .iter()
        .filter(|(code, _, _)| code == "match.opponent_ready")
        .map(|(_, to, _)| to)
        .collect();
    assert_eq!(asked, [&vec![other_cap.id]]);
    // O colega não é cobrado: o lado dele já está pronto.
    let seen = detail(&c.app, mate, KEY).await;
    assert_eq!(seen["summary"]["task"], "await_opponent");

    let body = ok_ready(&c, other_cap).await;
    assert_eq!(body["veto"]["status"], "in_progress");
}

#[tokio::test]
async fn anything_else_needs_everyone_including_a_custom_mix() {
    let (db, _lease) = crate::db::test_db().await;
    let mix = rules("captain", "captain", "captain", "anyone");
    let c = cast(&db, mix, draft("alternating", 1, 1)).await;
    let [cap, mate, other_cap, other_mate] = [&c.p[0], &c.p[1], &c.p[2], &c.p[3]];
    let seen = detail(&c.app, cap, KEY).await;
    assert_eq!(seen["veto"]["ready_by"], "everyone");
    assert_eq!(
        users(&seen["veto"]["ready_waiting"]),
        [cap.id, mate.id, other_cap.id, other_mate.id]
    );
    let body = ok_ready(&c, mate).await;
    assert_eq!(body["veto"]["ready_a"], false);
    assert_eq!(body["veto"]["can_act"]["ready"], false);
    assert_eq!(body["summary"]["task"], "await_team");
    // Repetir não muda nada.
    let again = ok_ready(&c, mate).await;
    assert_eq!(again["veto"]["history"].as_array().unwrap().len(), 1);
    // Faltam: o aviso do adversário só sai com o lado pronto.
    clear(&db).await;
    let body = ok_ready(&c, cap).await;
    assert_eq!(body["veto"]["ready_a"], true);
    let sent = notices(&outbox(&db).await);
    let asked: Vec<&Vec<i64>> = sent
        .iter()
        .filter(|(code, _, _)| code == "match.opponent_ready")
        .map(|(_, to, _)| to)
        .collect();
    assert_eq!(asked, [&ids(&[other_cap, other_mate])]);
    let body = ok_ready(&c, other_cap).await;
    assert_eq!(body["veto"]["status"], "waiting_ready");
    assert_eq!(users(&body["veto"]["ready_waiting"]), [other_mate.id]);
    assert_eq!(body["veto"]["ready_waiting"][0]["nickname"], "P4");
    let body = ok_ready(&c, other_mate).await;
    assert_eq!(body["veto"]["status"], "in_progress");
    assert_eq!(body["veto"]["ready_waiting"], json!([]));
}

/// Quem cansou de esperar desfaz o "Pronto": o lado deixa de estar pronto, e
/// marcar de novo não avisa o adversário outra vez.
#[tokio::test]
async fn whoever_marked_unmarks_until_the_veto_begins() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cast(&db, all_players(), draft("alternating", 1, 1)).await;
    let [cap, mate, other_cap, other_mate] = [&c.p[0], &c.p[1], &c.p[2], &c.p[3]];
    let warned = |sent: Vec<(String, Vec<i64>, Value)>| {
        sent.iter()
            .filter(|(code, _, _)| code == "match.opponent_ready")
            .count()
    };
    ok_ready(&c, cap).await;
    // Sem marca, desfazer não muda nada, nem para quem não joga a série.
    let body = ok_unready(&c, mate).await;
    assert_eq!(body["veto"]["can_act"]["unready"], false);
    assert_eq!(body["veto"]["history"].as_array().unwrap().len(), 1);
    assert_eq!(
        unready(&c, &c.ana).await,
        (StatusCode::FORBIDDEN, err(NOT_PLAYER))
    );
    clear(&db).await;
    let body = ok_ready(&c, mate).await;
    assert_eq!(body["veto"]["ready_a"], true);
    assert_eq!(body["veto"]["can_act"]["unready"], true);
    assert_eq!(warned(notices(&outbox(&db).await)), 1);

    clear(&db).await;
    let body = ok_unready(&c, mate).await;
    assert_eq!(body["veto"]["ready_a"], false);
    assert_eq!(body["veto"]["can_act"]["ready"], true);
    assert_eq!(body["veto"]["can_act"]["unready"], false);
    assert_eq!(users(&body["veto"]["ready_waiting"])[0], mate.id);
    assert_eq!(body["summary"]["task"], "ready");
    assert_eq!(last_event(&body)["kind"], "unready");
    assert_eq!(last_event(&body)["actor"], "P2");
    assert_eq!(warned(notices(&outbox(&db).await)), 0);
    // O colega continua marcado; o lado volta a ficar pronto sem novo aviso.
    let seen = detail(&c.app, cap, KEY).await;
    assert_eq!(seen["veto"]["can_act"]["unready"], true);
    assert_eq!(seen["summary"]["task"], "await_team");
    let body = ok_ready(&c, mate).await;
    assert_eq!(body["veto"]["ready_a"], true);
    assert_eq!(warned(notices(&outbox(&db).await)), 0);

    ok_ready(&c, other_cap).await;
    let body = ok_ready(&c, other_mate).await;
    assert_eq!(body["veto"]["status"], "in_progress");
    assert_eq!(body["veto"]["can_act"]["unready"], false);
    assert_eq!(
        unready(&c, mate).await,
        (StatusCode::CONFLICT, err("veto.already_begun"))
    );
}

#[tokio::test]
async fn a_substitute_has_to_mark_and_the_one_who_left_no_longer_counts() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cast(&db, all_players(), draft("alternating", 1, 1)).await;
    let [cap, mate, other_cap, other_mate] = [&c.p[0], &c.p[1], &c.p[2], &c.p[3]];
    // O capitão marca e sai do time; o novo ainda não marcou, e o outro lado está pronto.
    ok_ready(&c, cap).await;
    ok_ready(&c, other_cap).await;
    ok_ready(&c, other_mate).await;
    let substitution = format!("{COPA}/teams/{}/substitution", c.teams[0]);
    let (status, body) = post(
        &c.app,
        &substitution,
        &c.ana,
        json!({ "out": cap.id, "in": c.bia.id }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let seen = detail(&c.app, mate, KEY).await;
    assert_eq!(users(&seen["veto"]["ready_waiting"]), [mate.id, c.bia.id]);
    assert_eq!(seen["veto"]["status"], "waiting_ready");
    // Quem saiu não joga mais.
    assert_eq!(
        ready(&c, cap).await,
        (StatusCode::FORBIDDEN, err(NOT_PLAYER))
    );
    assert_eq!(
        detail(&c.app, &c.bia, KEY).await["summary"]["task"],
        "ready"
    );
    ok_ready(&c, mate).await;
    let body = ok_ready(&c, &c.bia).await;
    assert_eq!(body["veto"]["status"], "in_progress");
}

#[tokio::test]
async fn whoever_leaves_the_community_does_not_hold_the_series_forever() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cast(&db, all_players(), draft("alternating", 1, 1)).await;
    let [cap, mate, other_cap, other_mate] = [&c.p[0], &c.p[1], &c.p[2], &c.p[3]];
    ok_ready(&c, cap).await;
    ok_ready(&c, other_cap).await;
    ok_ready(&c, other_mate).await;
    // Só falta P2, que sai: ninguém mais vai clicar, e a série começa sozinha.
    let (status, body) = delete(&c.app, "/tenants/quake-br/me", mate).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    let seen = detail(&c.app, cap, KEY).await;
    assert_eq!(seen["veto"]["status"], "in_progress", "{seen}");
    assert_eq!(seen["veto"]["turn"]["action"], "ban");
}

/// Sem ninguém ler, quem começa é o relógio: a saída não grava evento de veto,
/// mas a mudança da comunidade o faz reler os torneios dela, em vez de confiar
/// no que viu antes (a série esperando quem saiu).
#[tokio::test]
async fn the_clock_begins_the_series_that_a_departure_left_ready() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cast(&db, all_players(), draft("alternating", 1, 1)).await;
    let [cap, mate, other_cap, other_mate] = [&c.p[0], &c.p[1], &c.p[2], &c.p[3]];
    ok_ready(&c, cap).await;
    ok_ready(&c, other_cap).await;
    ok_ready(&c, other_mate).await;
    const BEGUN: &str = "select count(*) from match_veto_events where kind = 'begin'";
    let now = chrono::Utc::now;
    assert_eq!(veto_clock(&c.state, now()).await.unwrap(), 0);
    let (status, body) = delete(&c.app, "/tenants/quake-br/me", mate).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    assert_eq!(count(&db, BEGUN).await, 0);
    assert_eq!(veto_clock(&c.state, now()).await.unwrap(), 0);
    assert_eq!(count(&db, BEGUN).await, 1);
}

#[tokio::test]
async fn a_new_captain_marks_and_the_old_mark_comes_back_with_him() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cast(&db, all_captain(), draft("alternating", 1, 1)).await;
    let [cap, mate, other_cap] = [&c.p[0], &c.p[1], &c.p[2]];
    let captain = |who: &User| json!({ "captain": who.id });
    let team = format!("{COPA}/teams/{}", c.teams[0]);
    ok_ready(&c, cap).await;
    let (status, body) = patch(&c.app, &team, &c.ana, captain(mate)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // O lado volta a esperar o capitão de agora.
    let seen = detail(&c.app, mate, KEY).await;
    assert_eq!(seen["veto"]["ready_a"], false);
    assert_eq!(seen["veto"]["can_act"]["ready"], true);
    assert_eq!(
        ready(&c, cap).await,
        (StatusCode::FORBIDDEN, err(NOT_CAPTAIN))
    );
    ok_ready(&c, other_cap).await;
    // Volta o capitão que já tinha marcado: os dois prontos, sem ninguém clicar.
    let (status, body) = patch(&c.app, &team, &c.ana, captain(cap)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let seen = detail(&c.app, cap, KEY).await;
    assert_eq!(seen["veto"]["status"], "in_progress", "{seen}");
}

#[tokio::test]
async fn a_begun_veto_asks_no_ready_even_with_people_missing() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cast(&db, all_players(), draft("alternating", 1, 1)).await;
    let [cap, mate] = [&c.p[0], &c.p[1]];
    // Um lado marcou pela metade (como a série de antes da regra), e o organizador
    // começa.
    ok_ready(&c, cap).await;
    let (status, body) = post_empty(&c.app, &uri("begin"), &c.ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let seen = detail(&c.app, mate, KEY).await;
    assert_eq!(seen["veto"]["ready_a"], true);
    assert_eq!(seen["veto"]["ready_b"], false);
    assert_eq!(seen["veto"]["ready_waiting"], json!([]));
    assert_eq!(seen["veto"]["can_act"]["ready"], false);
    assert_eq!(
        ready(&c, mate).await,
        (StatusCode::CONFLICT, err("veto.already_begun"))
    );
}
