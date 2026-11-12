//! Aceitar a contestação: só o capitão do time adversário de quem contestou,
//! qualquer que seja a regra de resultado, e dando o placar certo.

use super::*;

fn ok((status, body): (StatusCode, Value)) -> Value {
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

/// O lado `b` venceu, ao contrário do `game("a", map)` enviado.
fn flipped(map: &str) -> Value {
    json!({ "map": map, "a_frags": 5, "b_frags": 10 })
}

#[tokio::test]
async fn only_the_captain_of_the_other_team_accepts() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cast(&db, all_players(), single(3, 3)).await;
    let [cap, mate, other_cap, other_mate] = [&c.p[0], &c.p[1], &c.p[2], &c.p[3]];
    ok(report(&c.app, mate, KEY, 1, game("a", "awoken")).await);
    ok(contest(&c.app, other_mate, KEY, 1, "Foi 5 a 10").await);

    // A série pede a resposta dele, e a partida só dele.
    let seen = detail(&c.app, cap, KEY).await;
    assert_eq!(seen["games"][0]["task"], "accept_contest");
    assert_eq!(seen["summary"]["task"], "accept_contest");
    // O aviso diz quem enviou e quem contestou.
    let named = |d: &Value| {
        let t = &d["summary"]["task_detail"];
        (
            t["game"].clone(),
            t["reported_by"].clone(),
            t["contested_by"].clone(),
            t["viewer_side"].clone(),
        )
    };
    assert_eq!(
        named(&seen),
        (json!(1), json!("P2"), json!("P4"), json!("a"))
    );
    for who in [mate, other_cap, other_mate] {
        let seen = detail(&c.app, who, KEY).await;
        assert_eq!(seen["games"][0]["task"], "await_organizer");
    }
    let seen = detail(&c.app, &c.ana, KEY).await;
    assert_eq!(seen["games"][0]["task"], "resolve");
    assert_eq!(
        named(&seen),
        (json!(1), json!("P2"), json!("P4"), Value::Null)
    );

    let refused = (StatusCode::FORBIDDEN, err("match.not_accepter"));
    for who in [mate, other_cap, other_mate, &c.ana, &c.bia] {
        let tried = accept(&c.app, who, KEY, 1, flipped("awoken")).await;
        assert_eq!(tried, refused);
    }
    // O mesmo placar seria recusar a contestação sem passar pela organização.
    assert_eq!(
        accept(&c.app, cap, KEY, 1, game("a", "awoken")).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("match.contest_unchanged")
        )
    );

    clear(&db).await;
    let body = ok(accept(&c.app, cap, KEY, 1, flipped("awoken")).await);
    let g = &body["games"][0];
    assert_eq!(
        (&g["status"], &g["winner"], &g["a_frags"], &g["b_frags"]),
        (&json!("confirmed"), &json!("b"), &json!(5), &json!(10))
    );
    assert_eq!(
        (
            &g["confirmed_via"],
            &g["contest_comment"],
            &g["contested_by"]
        ),
        (&json!("opponent"), &Value::Null, &Value::Null)
    );
    assert_eq!(kinds(&body).last().unwrap(), "resolved:1");
    // Quem contestou fica sabendo, e ninguém mais.
    assert_eq!(
        notices(&outbox(&db).await),
        [(
            "match.contest_accepted".to_owned(),
            vec![other_mate.id],
            json!({ "tournament": "Copa de Outubro", "match": KEY, "game": 1, "player": "P1" })
        )]
    );
    assert_eq!(
        accept(&c.app, cap, KEY, 1, game("a", "awoken")).await,
        (StatusCode::CONFLICT, err("match.not_contested"))
    );

    // Se o placar dado também está errado, quem contestou contesta de novo.
    let seen = detail(&c.app, other_mate, KEY).await;
    assert_eq!(seen["games"][0]["can_contest"], true);
    assert_eq!(
        detail(&c.app, cap, KEY).await["games"][0]["can_contest"],
        false
    );
    let body = ok(contest(&c.app, other_mate, KEY, 1, "Foi 5 a 20").await);
    assert_eq!(body["games"][0]["status"], "contested");
    assert_eq!(
        detail(&c.app, cap, KEY).await["games"][0]["task"],
        "accept_contest"
    );
}

#[tokio::test]
async fn the_other_team_is_the_one_opposing_who_contested() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cast(&db, all_players(), single(3, 3)).await;
    let [cap, mate, other_cap, _] = [&c.p[0], &c.p[1], &c.p[2], &c.p[3]];
    // O colega discorda do próprio capitão: quem responde é o capitão de B.
    ok(report(&c.app, cap, KEY, 1, game("a", "awoken")).await);
    ok(contest(&c.app, mate, KEY, 1, "Foi o B").await);
    let task = |d: &Value| d["games"][0]["task"].clone();
    assert_eq!(task(&detail(&c.app, cap, KEY).await), "await_organizer");
    assert_eq!(
        task(&detail(&c.app, other_cap, KEY).await),
        "accept_contest"
    );
    let body = ok(accept(&c.app, other_cap, KEY, 1, flipped("awoken")).await);
    assert_eq!(body["games"][0]["reported_side"], "b");
}

#[tokio::test]
async fn the_captain_accepts_whatever_the_result_rule() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cast(&db, all_captain(), single(3, 3)).await;
    let [cap, mate, other_cap, _] = [&c.p[0], &c.p[1], &c.p[2], &c.p[3]];
    ok(report(&c.app, cap, KEY, 1, game("a", "awoken")).await);
    ok(contest(&c.app, other_cap, KEY, 1, "Foi o B").await);
    let refused = (StatusCode::FORBIDDEN, err("match.not_accepter"));
    assert_eq!(
        accept(&c.app, mate, KEY, 1, flipped("awoken")).await,
        refused
    );
    // Quem enviou pode reconhecer o erro.
    ok(accept(&c.app, cap, KEY, 1, flipped("awoken")).await);
}

#[tokio::test]
async fn the_map_stays_the_one_the_veto_gave() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cast(&db, all_players(), map_veto()).await;
    let [cap, _, other_cap, _] = [&c.p[0], &c.p[1], &c.p[2], &c.p[3]];
    all_ready(&c).await;
    let map = loop {
        let seen = detail(&c.app, cap, KEY).await;
        if seen["veto"]["status"] == "done" {
            break seen["veto"]["games"][0]["map"]["slug"]
                .as_str()
                .unwrap()
                .to_owned();
        }
        let who = if seen["veto"]["turn"]["sides"] == json!(["a"]) {
            cap
        } else {
            other_cap
        };
        let options = detail(&c.app, who, KEY).await["veto"]["options"][0].clone();
        ok_step(&c, who, json!({ "map": options })).await;
    };
    ok(report(&c.app, cap, KEY, 1, game("a", &map)).await);
    ok(contest(&c.app, other_cap, KEY, 1, "Foi o B").await);

    let other = if map == "awoken" {
        "vestibule-of-exile"
    } else {
        "awoken"
    };
    assert_eq!(
        accept(&c.app, cap, KEY, 1, flipped(other)).await,
        (StatusCode::UNPROCESSABLE_ENTITY, err("match.map_mismatch"))
    );
    ok(accept(&c.app, cap, KEY, 1, flipped(&map)).await);
}
