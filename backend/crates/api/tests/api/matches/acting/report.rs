//! Enviar, confirmar e contestar × a regra de resultado.

use super::*;

// Enviar, confirmar e contestar.

/// `by` envia a partida 1 (sem fases no torneio).
async fn sent(c: &Cast, by: &User) -> Value {
    let (status, body) = report(&c.app, by, KEY, 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

async fn no_phases(db: &PgPool, report_by: &str) -> Cast {
    cast(
        db,
        rules("anyone", "anyone", "each_player", report_by),
        single(3, 3),
    )
    .await
}

#[tokio::test]
async fn reporting_by_the_captain() {
    let (db, _lease) = crate::db::test_db().await;
    let c = no_phases(&db, "captain").await;
    let [cap, mate, other_cap, other_mate] = [&c.p[0], &c.p[1], &c.p[2], &c.p[3]];
    let not_captain = (StatusCode::FORBIDDEN, err(NOT_CAPTAIN));
    let not_player = (StatusCode::FORBIDDEN, err(NOT_PLAYER));
    let send = |who| report(&c.app, who, KEY, 1, game("a", "awoken"));
    assert_eq!(send(mate).await, not_captain);
    assert_eq!(send(other_mate).await, not_captain);
    assert_eq!(send(&c.ana).await, not_player);
    assert_eq!(send(&c.bia).await, not_player);
    let seen = detail(&c.app, mate, KEY).await;
    assert_eq!(seen["can_report"], false);
    assert_eq!(seen["summary"]["task"], "await_captain");
    let seen = detail(&c.app, cap, KEY).await;
    assert_eq!(seen["can_report"], true);
    assert_eq!(seen["summary"]["task"], "report");
    // O capitão de qualquer lado envia.
    clear(&db).await;
    let body = sent(&c, other_cap).await;
    assert_eq!(body["games"][0]["reported_side"], "b");
    // O aviso pede resposta: vai só para quem pode dar.
    let sent_to: Vec<Vec<i64>> = notices(&outbox(&db).await)
        .into_iter()
        .filter(|(code, _, _)| code == "match.game_reported")
        .map(|(_, to, _)| to)
        .collect();
    assert_eq!(sent_to, [vec![cap.id]]);
}

#[tokio::test]
async fn reporting_by_anyone() {
    let (db, _lease) = crate::db::test_db().await;
    let c = no_phases(&db, "anyone").await;
    let [cap, mate, other_cap, other_mate] = [&c.p[0], &c.p[1], &c.p[2], &c.p[3]];
    assert_eq!(detail(&c.app, mate, KEY).await["summary"]["task"], "report");
    clear(&db).await;
    sent(&c, mate).await;
    let sent_to: Vec<Vec<i64>> = notices(&outbox(&db).await)
        .into_iter()
        .filter(|(code, _, _)| code == "match.game_reported")
        .map(|(_, to, _)| to)
        .collect();
    assert_eq!(sent_to, [ids(&[other_cap, other_mate])]);
    // O colega responde, e o capitão de quem enviou contesta.
    assert_eq!(
        detail(&c.app, other_mate, KEY).await["summary"]["task"],
        "respond"
    );
    let (status, body) = contest(&c.app, cap, KEY, 1, "foi 9").await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
async fn confirming_by_the_captain() {
    let (db, _lease) = crate::db::test_db().await;
    let c = no_phases(&db, "captain").await;
    let [cap, mate, other_cap, other_mate] = [&c.p[0], &c.p[1], &c.p[2], &c.p[3]];
    sent(&c, cap).await;
    assert_eq!(
        confirm(&c.app, other_mate, KEY, 1).await,
        (StatusCode::FORBIDDEN, err(NOT_CAPTAIN))
    );
    // Do time de quem enviou, nem o capitão.
    let own = (StatusCode::FORBIDDEN, err("match.own_report"));
    assert_eq!(confirm(&c.app, mate, KEY, 1).await, own);
    assert_eq!(confirm(&c.app, cap, KEY, 1).await, own);
    assert_eq!(
        confirm(&c.app, &c.bia, KEY, 1).await,
        (StatusCode::FORBIDDEN, err(NOT_PLAYER))
    );
    let seen = detail(&c.app, other_mate, KEY).await;
    assert_eq!(seen["games"][0]["task"], "await_captain");
    assert_eq!(seen["games"][0]["can_contest"], false);
    assert_eq!(seen["summary"]["task"], "await_captain");
    let seen = detail(&c.app, other_cap, KEY).await;
    assert_eq!(seen["games"][0]["task"], "respond");
    assert_eq!(seen["summary"]["task"], "respond");
    let (status, body) = confirm(&c.app, other_cap, KEY, 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
async fn the_organizer_confirms_whatever_the_rule() {
    let (db, _lease) = crate::db::test_db().await;
    let c = no_phases(&db, "captain").await;
    sent(&c, &c.p[0]).await;
    let (status, body) = confirm(&c.app, &c.ana, KEY, 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["games"][0]["confirmed_via"], "organizer");
}

#[tokio::test]
async fn contesting_by_the_captain() {
    let (db, _lease) = crate::db::test_db().await;
    let c = no_phases(&db, "captain").await;
    let [cap, mate, other_cap, other_mate] = [&c.p[0], &c.p[1], &c.p[2], &c.p[3]];
    sent(&c, cap).await;
    let contesting = |who| contest(&c.app, who, KEY, 1, "foi 9");
    let not_captain = (StatusCode::FORBIDDEN, err(NOT_CAPTAIN));
    assert_eq!(contesting(other_mate).await, not_captain);
    assert_eq!(contesting(mate).await, not_captain);
    assert_eq!(
        contesting(cap).await,
        (StatusCode::FORBIDDEN, err("match.own_report"))
    );
    assert_eq!(
        contesting(&c.ana).await,
        (StatusCode::FORBIDDEN, err(NOT_PLAYER))
    );
    assert_eq!(
        detail(&c.app, other_cap, KEY).await["games"][0]["can_contest"],
        true
    );
    let (status, body) = contesting(other_cap).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
async fn contesting_by_anyone() {
    let (db, _lease) = crate::db::test_db().await;
    let c = no_phases(&db, "anyone").await;
    let [cap, mate, other_mate] = [&c.p[0], &c.p[1], &c.p[3]];
    sent(&c, cap).await;
    assert_eq!(
        detail(&c.app, other_mate, KEY).await["games"][0]["can_contest"],
        true
    );
    // O colega de quem enviou também discorda.
    let (status, body) = contest(&c.app, mate, KEY, 1, "foi 9").await;
    assert_eq!(status, StatusCode::OK, "{body}");
}
