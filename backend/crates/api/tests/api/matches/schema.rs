//! Regras do banco para mapas, séries e histórico: estado inválido gravado
//! direto por SQL é recusado, com o código e a regra.

use serde_json::json;
use sqlx::PgPool;

use super::*;
use crate::common::*;
use crate::schema::{CHECK, FOREIGN_KEY, UNIQUE, exec, rejected, rule};

/// `(tenant_id, tournament_id, [time do seed 1, do seed 2])` do torneio
/// `slug`.
async fn ids(db: &PgPool, slug: &str) -> (i64, i64, Vec<i64>) {
    let t = sqlx::query!(
        "select tenant_id, id from tournaments where slug = $1",
        slug
    )
    .fetch_one(db)
    .await
    .unwrap();
    let seeded = sqlx::query_scalar!(
        "select id from teams where tournament_id = $1 and seed is not null order by seed",
        t.id
    )
    .fetch_all(db)
    .await
    .unwrap();
    (t.tenant_id, t.id, seeded)
}

/// Um mapa de `wb-1-1` com os campos que variam nos testes (enviado pelo
/// lado `a`; contestado com `comment`; confirmado pelo adversário).
struct Game<'a> {
    sides: (i64, i64),
    number: i16,
    winner: i64,
    map: Option<&'a str>,
    frags: (Option<i16>, Option<i16>),
    status: &'a str,
    comment: Option<&'a str>,
}

async fn insert_game(
    db: &PgPool,
    (tenant, tournament): (i64, i64),
    g: Game<'_>,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query(
        r#"insert into match_games
               (tenant_id, tournament_id, stage, match_key, number, side_a, side_b, winner,
                map_id, a_frags, b_frags, status, reported_side, confirm_by, contested_at,
                contest_comment, confirmed_via, confirmed_at)
           values ($1, $2, 1, 'wb-1-1', $3, $4, $5, $6, (select id from maps where slug = $7),
                   $8, $9, $10, 'a', now() + interval '10 minutes',
                   case when $10 = 'contested' then now() end, $11,
                   case when $10 = 'confirmed' then 'opponent' end,
                   case when $10 = 'confirmed' then now() end)"#,
    )
    .bind(tenant)
    .bind(tournament)
    .bind(g.number)
    .bind(g.sides.0)
    .bind(g.sides.1)
    .bind(g.winner)
    .bind(g.map)
    .bind(g.frags.0)
    .bind(g.frags.1)
    .bind(g.status)
    .bind(g.comment)
    .execute(db)
    .await
}

/// A série de `wb-1-1` decidida pelos mapas para `winner`, com `wins`.
async fn insert_series(
    db: &PgPool,
    (tenant, tournament): (i64, i64),
    (a, b): (i64, i64),
    winner: i64,
    wins: (i16, i16),
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query(
        r#"insert into match_results
               (tenant_id, tournament_id, stage, match_key, best_of, side_a, side_b, status,
                winner, a_wins, b_wins, confirmed_via, confirmed_at, position)
           values ($1, $2, 1, 'wb-1-1', 3, $3, $4, 'confirmed', $5, $6, $7, 'maps', now(), 1)"#,
    )
    .bind(tenant)
    .bind(tournament)
    .bind(a)
    .bind(b)
    .bind(winner)
    .bind(wins.0)
    .bind(wins.1)
    .execute(db)
    .await
}

#[tokio::test]
async fn only_a_contested_map_waits_for_the_notice() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    let (status, body) = report(&app, &p[0], "wb-1-1", 1, game("a", "awoken")).await;
    assert_eq!(status, axum::http::StatusCode::OK, "{body}");
    assert_eq!(
        rejected(exec(&db, "update match_games set contest_notify_at = now()").await),
        rule(CHECK, "match_games_contest_notify")
    );
}

#[tokio::test]
async fn only_a_contested_map_has_a_proposal() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    let (status, body) = report(&app, &p[0], "wb-1-1", 1, game("a", "awoken")).await;
    assert_eq!(status, axum::http::StatusCode::OK, "{body}");
    let map = "contest_map_id = (select min(id) from maps)";
    let full = format!("update match_games set {map}, contest_a_frags = 1, contest_b_frags = 2");
    assert_eq!(
        rejected(exec(&db, &full).await),
        rule(CHECK, "match_games_contest_proposal_status")
    );
    let (status, body) = contest(&app, &p[1], "wb-1-1", 1, "Foi o B").await;
    assert_eq!(status, axum::http::StatusCode::OK, "{body}");
    for (set, name) in [
        (
            "contest_a_frags = 1".to_owned(),
            "match_games_contest_proposal",
        ),
        (
            format!("{map}, contest_a_frags = 1"),
            "match_games_contest_proposal",
        ),
        (
            format!("{map}, contest_a_frags = 7, contest_b_frags = 7"),
            "match_games_contest_proposal_tied",
        ),
        (
            format!("{map}, contest_a_frags = 1000, contest_b_frags = 0"),
            "match_games_contest_a_frags_check",
        ),
    ] {
        let sql = format!("update match_games set {set}");
        assert_eq!(rejected(exec(&db, &sql).await), rule(CHECK, name), "{set}");
    }
    exec(&db, &full).await.unwrap();
}

#[tokio::test]
async fn maps_must_be_valid() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    let (tenant, tournament, seeded) = ids(&db, "copa").await;
    let t = (tenant, tournament);
    let (a, b) = (seeded[0], seeded[1]);
    let base = || Game {
        sides: (a, b),
        number: 1,
        winner: a,
        map: Some("awoken"),
        // Suicídio e queda tiram frag: o placar pode ficar abaixo de 0.
        frags: (Some(20), Some(-1)),
        status: "pending",
        comment: None,
    };
    let check = |name: &str| rule(CHECK, name);

    let cases = [
        (
            Game {
                frags: (Some(10), Some(10)),
                ..base()
            },
            check("match_games_frags_winner"),
        ),
        (
            Game {
                frags: (Some(5), Some(10)),
                ..base()
            },
            check("match_games_frags_winner"),
        ),
        (
            Game {
                frags: (Some(-3), Some(-1)),
                ..base()
            },
            check("match_games_frags_winner"),
        ),
        (
            Game {
                frags: (Some(-999), Some(-1000)),
                ..base()
            },
            check("match_games_b_frags_check"),
        ),
        (
            Game {
                frags: (Some(5), None),
                ..base()
            },
            check("match_games_frags"),
        ),
        (
            Game {
                map: None,
                ..base()
            },
            check("match_games_frags_map"),
        ),
        (
            Game {
                sides: (a, a),
                ..base()
            },
            check("match_games_sides"),
        ),
        (
            Game {
                winner: a + b + 1,
                frags: (None, None),
                ..base()
            },
            check("match_games_winner"),
        ),
        (
            Game {
                status: "contested",
                ..base()
            },
            check("match_games_contest_comment"),
        ),
        (
            Game {
                status: "done",
                ..base()
            },
            check("match_games_status_check"),
        ),
        (
            Game {
                number: 0,
                ..base()
            },
            check("match_games_number_check"),
        ),
        // O primeiro mapa é o 1, sem buraco (conferido no commit).
        (
            Game {
                number: 2,
                ..base()
            },
            check("match_games_sequence"),
        ),
    ];
    for (game, expected) in cases {
        assert_eq!(rejected(insert_game(&db, t, game).await), expected);
    }

    // O que vale passa: com frags, ou sem mapa nem frags (série antiga).
    insert_game(&db, t, base()).await.unwrap();
    let old = Game {
        number: 2,
        map: None,
        frags: (None, None),
        ..base()
    };
    insert_game(&db, t, old).await.unwrap();
    // O mapa não se repete e o número não se repete.
    let repeated = Game {
        number: 3,
        ..base()
    };
    assert_eq!(
        rejected(insert_game(&db, t, repeated).await),
        rule(
            UNIQUE,
            "match_games_tournament_id_stage_match_key_map_id_key"
        )
    );
    let again = Game {
        map: Some("vestibule-of-exile"),
        ..base()
    };
    assert_eq!(
        rejected(insert_game(&db, t, again).await),
        rule(
            UNIQUE,
            "match_games_tournament_id_stage_match_key_number_key"
        )
    );
    // Todos os mapas da partida têm os mesmos lados.
    let flipped = Game {
        sides: (b, a),
        number: 3,
        map: Some("vestibule-of-exile"),
        frags: (Some(1), Some(9)),
        ..base()
    };
    assert_eq!(
        rejected(insert_game(&db, t, flipped).await),
        check("match_games_sequence")
    );
    // Tirar um mapa do meio deixa buraco.
    assert_eq!(
        rejected(exec_sql(&db, "delete from match_games where number = 1").await),
        check("match_games_sequence")
    );
}

#[tokio::test]
async fn the_decided_series_agrees_with_its_maps() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    let (tenant, tournament, seeded) = ids(&db, "copa").await;
    let t = (tenant, tournament);
    let (a, b) = (seeded[0], seeded[1]);
    for (number, map) in [(1, "awoken"), (2, "vestibule-of-exile")] {
        let g = Game {
            sides: (a, b),
            number,
            winner: a,
            map: Some(map),
            frags: (None, None),
            status: "confirmed",
            comment: None,
        };
        insert_game(&db, t, g).await.unwrap();
    }

    // Placar diferente do dos mapas.
    assert_eq!(
        rejected(insert_series(&db, t, (a, b), b, (0, 2)).await),
        rule(CHECK, "match_results_games_score")
    );
    insert_series(&db, t, (a, b), a, (2, 0)).await.unwrap();
    // Série decidida não tem mapa pendente.
    let pending = Game {
        sides: (a, b),
        number: 3,
        winner: b,
        map: Some("insomnia"),
        frags: (None, None),
        status: "pending",
        comment: None,
    };
    assert_eq!(
        rejected(insert_game(&db, t, pending).await),
        rule(CHECK, "match_games_series_decided")
    );
    // Só a série decidida fica em match_results, e a decidida pelos mapas
    // não tem quem enviou.
    let pending = "update match_results set status = 'pending', confirmed_via = null,
                   confirmed_at = null, position = null";
    assert_eq!(
        rejected(exec_sql(&db, pending).await),
        rule(CHECK, "match_results_decided")
    );
    let reported = "update match_results set reported_side = 'a',
                    confirm_by = reported_at + interval '1 minute'";
    assert_eq!(
        rejected(exec_sql(&db, reported).await),
        rule(CHECK, "match_results_maps_check")
    );

    // Com mapas, o time não sai (a chave depende dele).
    assert_eq!(
        rejected(
            sqlx::query("delete from teams where id = $1")
                .bind(b)
                .execute(&db)
                .await
        )
        .0,
        FOREIGN_KEY
    );
}

#[tokio::test]
async fn maps_stay_inside_their_tournament() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    let (tenant, tournament, seeded) = ids(&db, "copa").await;
    let a = seeded[0];

    // Time de outro torneio do mesmo tenant.
    let outra = with(duel("outra"), json!({ "check_in": false }));
    create_tournament(&app, &ana, outra).await;
    publish(&app, &ana, "outra").await;
    register(&app, "outra", &[&p[0]]).await;
    let other = sqlx::query_scalar!(
        "select tm.id from teams tm join tournaments t on t.id = tm.tournament_id
         where t.slug = 'outra'"
    )
    .fetch_one(&db)
    .await
    .unwrap();
    let g = Game {
        sides: (a, other),
        number: 1,
        winner: a,
        map: Some("awoken"),
        frags: (None, None),
        status: "pending",
        comment: None,
    };
    assert_eq!(
        rejected(insert_game(&db, (tenant, tournament), g).await).0,
        FOREIGN_KEY
    );
}

#[tokio::test]
async fn events_reject_invalid_rows() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    let (tenant, tournament, seeded) = ids(&db, "copa").await;
    let a = seeded[0];
    let event = |kind: &'static str,
                 actor: Option<i64>,
                 game: Option<i16>,
                 winner: Option<i64>,
                 maps: Option<(i16, i16)>,
                 comment: Option<&'static str>| {
        sqlx::query(
            "insert into match_events
                 (tenant_id, tournament_id, stage, match_key, kind, actor, game, winner,
                  winner_maps, loser_maps, comment)
             values ($1, $2, 1, 'wb-1-1', $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(tenant)
        .bind(tournament)
        .bind(kind)
        .bind(actor)
        .bind(game)
        .bind(winner)
        .bind(maps.map(|m| m.0))
        .bind(maps.map(|m| m.1))
        .bind(comment)
        .execute(&db)
    };
    let me = Some(p[0].id);
    let outcome = rule(CHECK, "match_events_outcome_check");

    // O que vale passa: mapa enviado (só o vencedor), série decidida, mapa
    // anulado para ser jogado de novo (sem vencedor).
    event("reported", me, Some(1), Some(a), None, None)
        .await
        .unwrap();
    event("decided", None, None, Some(a), Some((2, 1)), None)
        .await
        .unwrap();
    event("replay", me, Some(1), None, None, None)
        .await
        .unwrap();
    // Mapa com placar de série, mapa enviado sem vencedor, confirmação com
    // vencedor, série fechada sem placar ou com autor, W.O. num mapa, mapa
    // anulado com vencedor ou sem o mapa.
    for (kind, game, winner, maps) in [
        ("reported", Some(1), Some(a), Some((2, 1))),
        ("reported", Some(1), None, None),
        ("confirmed", Some(1), Some(a), None),
        ("decided", None, Some(a), None),
        ("walkover", Some(1), Some(a), None),
        ("replay", Some(1), Some(a), None),
        ("replay", None, None, None),
    ] {
        assert_eq!(
            rejected(event(kind, None, game, winner, maps, None).await),
            outcome,
            "{kind} {game:?}"
        );
    }
    assert_eq!(
        rejected(event("decided", me, None, Some(a), Some((2, 0)), None).await),
        rule(CHECK, "match_events_decided_actor")
    );
    assert_eq!(
        rejected(event("contested", me, Some(1), None, None, None).await).0,
        CHECK
    ); // sem comentário
    assert_eq!(
        rejected(event("undone", me, None, None, None, None).await),
        rule(CHECK, "match_events_kind_check")
    );
}

/// A série `key` em md`best_of`, sem autor, com o vencedor e o placar dados e
/// `confirmed_via` conforme o placar (os mapas ou o organizador).
async fn insert_decided(
    db: &PgPool,
    (tenant, tournament): (i64, i64),
    (a, b): (i64, i64),
    (key, best_of): (&str, i16),
    winner: Option<i64>,
    wins: Option<(i16, i16)>,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query(
        r#"insert into match_results
               (tenant_id, tournament_id, stage, match_key, best_of, side_a, side_b, status,
                winner, a_wins, b_wins, confirmed_via, confirmed_at, position)
           values ($1, $2, 1, $3, $4, $5, $6, 'confirmed', $7, $8, $9,
                   case when $8::smallint is null then 'organizer' else 'maps' end, now(), 1)"#,
    )
    .bind(tenant)
    .bind(tournament)
    .bind(key)
    .bind(best_of)
    .bind(a)
    .bind(b)
    .bind(winner)
    .bind(wins.map(|w| w.0))
    .bind(wins.map(|w| w.1))
    .execute(db)
    .await
}

#[tokio::test]
async fn only_an_even_series_in_the_round_robin_draws() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    let body = copa(table("round_robin", json!({ "best_of": 2 })));
    start(&app, &ana, body, &[&p[0], &p[1]]).await;
    let (tenant, tournament, seeded) = ids(&db, "copa").await;
    let t = (tenant, tournament);
    let sides = (seeded[0], seeded[1]);
    let a = seeded[0];
    let cases = [
        // Sem vencedor, só metade para cada lado numa série par: nem na md3, nem
        // outro placar, nem o W.O.
        (("rr-1-1", 3), None, Some((1, 1)), "match_results_draw"),
        (("rr-1-1", 2), None, Some((2, 0)), "match_results_draw"),
        (("rr-1-1", 2), None, None, "match_results_draw"),
        (("rr-1-1", 4), None, Some((1, 1)), "match_results_draw"),
        (("rr-1-1", 4), None, Some((3, 1)), "match_results_draw"),
        (("rr-1-1", 6), None, Some((2, 2)), "match_results_draw"),
        // Com vencedor, a md2 é 2 x 0: 1 x 1 não tem vencedor, e 2 x 1 são 3 mapas.
        (
            ("rr-1-1", 2),
            Some(a),
            Some((1, 1)),
            "match_results_score_check",
        ),
        (
            ("rr-1-1", 2),
            Some(a),
            Some((2, 1)),
            "match_results_score_check",
        ),
        (
            ("rr-1-1", 2),
            Some(a),
            Some((0, 2)),
            "match_results_score_check",
        ),
        // Na md4 não há 3 x 2 nem vencedor em 2 x 2; na md6, nem 4 x 3 nem 3 x 3.
        (
            ("rr-1-1", 4),
            Some(a),
            Some((3, 2)),
            "match_results_score_check",
        ),
        (
            ("rr-1-1", 4),
            Some(a),
            Some((2, 2)),
            "match_results_score_check",
        ),
        (
            ("rr-1-1", 6),
            Some(a),
            Some((4, 3)),
            "match_results_score_check",
        ),
        (
            ("rr-1-1", 6),
            Some(a),
            Some((3, 3)),
            "match_results_score_check",
        ),
        // A série par é só do round robin; a md8 também tem maioria (5).
        (
            ("wb-1-1", 2),
            Some(a),
            Some((2, 0)),
            "match_results_best_of_check",
        ),
        (
            ("sw-1-1", 2),
            Some(a),
            Some((2, 0)),
            "match_results_best_of_check",
        ),
        (
            ("wb-1-1", 4),
            Some(a),
            Some((3, 0)),
            "match_results_best_of_check",
        ),
        (
            ("sw-1-1", 6),
            Some(a),
            Some((4, 0)),
            "match_results_best_of_check",
        ),
        (
            ("rr-1-1", 8),
            Some(a),
            Some((4, 4)),
            "match_results_score_check",
        ),
    ];
    for (series, winner, wins, name) in cases {
        assert_eq!(
            rejected(insert_decided(&db, t, sides, series, winner, wins).await),
            rule(CHECK, name),
            "{series:?} {winner:?} {wins:?}"
        );
    }

    // O W.O. da md2 tem vencedor e não tem placar.
    insert_decided(&db, t, sides, ("rr-1-1", 2), Some(a), None)
        .await
        .unwrap();
}

#[tokio::test]
async fn a_draw_in_the_history_has_the_same_maps_each_and_no_winner() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    let body = copa(table("round_robin", json!({ "best_of": 2 })));
    start(&app, &ana, body, &[&p[0], &p[1]]).await;
    let (tenant, tournament, seeded) = ids(&db, "copa").await;
    let a = seeded[0];
    let event =
        |kind: &'static str, key: &'static str, winner: Option<i64>, maps: Option<(i16, i16)>| {
            sqlx::query(
                "insert into match_events
                 (tenant_id, tournament_id, stage, match_key, kind, winner, winner_maps, loser_maps)
             values ($1, $2, 1, $3, $4, $5, $6, $7)",
            )
            .bind(tenant)
            .bind(tournament)
            .bind(key)
            .bind(kind)
            .bind(winner)
            .bind(maps.map(|m| m.0))
            .bind(maps.map(|m| m.1))
            .execute(&db)
        };
    for tie in [(1, 1), (2, 2), (3, 3), (4, 4)] {
        event("decided", "rr-1-1", None, Some(tie)).await.unwrap();
    }
    event("decided", "rr-1-1", Some(a), Some((2, 0)))
        .await
        .unwrap();
    let maps = rule(CHECK, "match_events_maps_check");
    for (kind, key, winner, score) in [
        // Sem vencedor, só o mesmo número para cada lado, pelo menos 1, só ao decidir,
        // só no round robin.
        ("decided", "rr-1-1", None, Some((2, 0))),
        ("decided", "rr-1-1", None, Some((2, 1))),
        ("decided", "rr-1-1", None, Some((0, 0))),
        ("decided", "wb-1-1", None, Some((1, 1))),
        ("corrected", "rr-1-1", None, Some((1, 1))),
        // Com vencedor, 1 x 1 não existe.
        ("decided", "rr-1-1", Some(a), Some((1, 1))),
    ] {
        assert_eq!(
            rejected(event(kind, key, winner, score).await),
            maps,
            "{kind} {key} {winner:?} {score:?}"
        );
    }
    // Decidida sem placar, e não há W.O. empatado.
    let outcome = rule(CHECK, "match_events_outcome_check");
    assert_eq!(
        rejected(event("decided", "rr-1-1", None, None).await),
        outcome
    );
    assert_eq!(
        rejected(event("walkover", "rr-1-1", None, None).await),
        outcome
    );
}

async fn exec_sql(db: &PgPool, sql: &'static str) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query(sql).execute(db).await?.rows_affected())
}
