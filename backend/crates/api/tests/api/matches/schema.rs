//! Regras do banco para resultados, mapas e histórico: estado inválido
//! gravado direto por SQL é recusado.

use serde_json::json;
use sqlx::PgPool;

use super::*;
use crate::common::*;

const CHECK: &str = "23514";
const FOREIGN_KEY: &str = "23503";
const UNIQUE: &str = "23505";

/// Código do Postgres do erro (ex.: `23514` para check).
fn code(result: Result<sqlx::postgres::PgQueryResult, sqlx::Error>) -> String {
    let err = result.expect_err("o banco devia recusar");
    err.as_database_error()
        .and_then(|e| e.code())
        .map(|c| c.into_owned())
        .unwrap_or_else(|| panic!("não é erro do banco: {err}"))
}

/// `(tenant_id, tournament_id, [participante do seed 1, do seed 2])` do
/// torneio `slug`.
async fn ids(db: &PgPool, slug: &str) -> (i64, i64, Vec<i64>) {
    let t = sqlx::query!(
        "select tenant_id, id from tournaments where slug = $1",
        slug
    )
    .fetch_one(db)
    .await
    .unwrap();
    let seeded = sqlx::query_scalar!(
        "select id from participants where tournament_id = $1 and seed is not null order by seed",
        t.id
    )
    .fetch_all(db)
    .await
    .unwrap();
    (t.tenant_id, t.id, seeded)
}

/// Insere um resultado em `wb-1-1` (md3) com os campos que variam nos testes.
#[allow(clippy::too_many_arguments)]
async fn insert_result(
    db: &PgPool,
    (tenant, tournament): (i64, i64),
    (a, b): (i64, i64),
    status: &str,
    winner: i64,
    wins: (Option<i16>, Option<i16>),
    contest_comment: Option<&str>,
    position: Option<i32>,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query(
        r#"insert into match_results
               (tenant_id, tournament_id, stage, match_key, best_of, side_a, side_b, status,
                winner, a_wins, b_wins, reported_side, confirm_by, contested_at, contest_comment,
                confirmed_via, confirmed_at, position)
           values ($1, $2, 1, 'wb-1-1', 3, $3, $4, $5, $6, $7, $8, 'a', now() + interval '5 minutes',
                   case when $5 = 'contested' then now() end, $9,
                   case when $5 = 'confirmed' then 'opponent' end,
                   case when $5 = 'confirmed' then now() end, $10)"#,
    )
    .bind(tenant)
    .bind(tournament)
    .bind(a)
    .bind(b)
    .bind(status)
    .bind(winner)
    .bind(wins.0)
    .bind(wins.1)
    .bind(contest_comment)
    .bind(position)
    .execute(db)
    .await
}

#[sqlx::test(migrations = "../../migrations")]
async fn results_must_be_a_valid_series_and_state(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    let (tenant, tournament, seeded) = ids(&db, "copa").await;
    let t = (tenant, tournament);
    let (a, b) = (seeded[0], seeded[1]);
    let won = (Some(2), Some(1));

    // O que vale passa.
    insert_result(&db, t, (a, b), "pending", a, won, None, None)
        .await
        .unwrap();
    sqlx::query("delete from match_results")
        .execute(&db)
        .await
        .unwrap();

    for (wins, winner) in [
        ((Some(2), Some(2)), a), // empate
        ((Some(3), Some(0)), a), // além do melhor de 3
        ((Some(1), Some(2)), a), // vencedor com menos mapas
        ((Some(2), None), a),    // meio placar
    ] {
        assert_eq!(
            code(insert_result(&db, t, (a, b), "pending", winner, wins, None, None).await),
            CHECK,
            "{wins:?}"
        );
    }
    // Contestado sem comentário, confirmado sem posição no replay e status
    // fora da lista.
    for status in ["contested", "confirmed", "done"] {
        assert_eq!(
            code(insert_result(&db, t, (a, b), status, a, won, None, None).await),
            CHECK,
            "{status}"
        );
    }
    // O mesmo dos dois lados; vencedor que não joga a partida.
    assert_eq!(
        code(insert_result(&db, t, (a, a), "pending", a, won, None, None).await),
        CHECK
    );
    assert_eq!(
        code(insert_result(&db, t, (a, b), "pending", a + b + 1, won, None, None).await),
        CHECK
    );

    // Um resultado por partida.
    insert_result(&db, t, (a, b), "pending", a, won, None, None)
        .await
        .unwrap();
    assert_eq!(
        code(insert_result(&db, t, (a, b), "pending", b, (Some(0), Some(2)), None, None).await),
        UNIQUE
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn results_stay_inside_their_tournament(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    let (tenant, tournament, seeded) = ids(&db, "copa").await;
    let (a, b) = (seeded[0], seeded[1]);

    // Participante de outro torneio do mesmo tenant.
    let outra = with(duel("outra"), json!({ "check_in_minutes": null }));
    create_tournament(&app, &ana, outra).await;
    publish(&app, &ana, "outra").await;
    register(&app, "outra", &[&p[0]]).await;
    let other = sqlx::query_scalar!(
        "select p.id from participants p join tournaments t on t.id = p.tournament_id
         where t.slug = 'outra'"
    )
    .fetch_one(&db)
    .await
    .unwrap();
    let won = (Some(2), Some(0));
    assert_eq!(
        code(
            insert_result(
                &db,
                (tenant, tournament),
                (a, other),
                "pending",
                a,
                won,
                None,
                None
            )
            .await
        ),
        FOREIGN_KEY
    );
    // Torneio de outro tenant.
    let zeca = user(&db, "Zeca").await;
    create(&app, &zeca, "outro-tenant").await;
    let foreign = sqlx::query_scalar!("select id from tenants where slug = 'outro-tenant'")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(
        code(
            insert_result(
                &db,
                (foreign, tournament),
                (a, b),
                "pending",
                a,
                won,
                None,
                None
            )
            .await
        ),
        FOREIGN_KEY
    );

    // Com resultado, o participante não sai (a chave depende dele).
    insert_result(
        &db,
        (tenant, tournament),
        (a, b),
        "pending",
        a,
        won,
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(
        code(
            sqlx::query("delete from participants where id = $1")
                .bind(b)
                .execute(&db)
                .await
        ),
        FOREIGN_KEY
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn games_and_events_reject_invalid_rows(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    let (tenant, tournament, seeded) = ids(&db, "copa").await;
    let (a, b) = (seeded[0], seeded[1]);
    insert_result(
        &db,
        (tenant, tournament),
        (a, b),
        "pending",
        a,
        (Some(2), Some(0)),
        None,
        None,
    )
    .await
    .unwrap();
    let result = sqlx::query_scalar!("select id from match_results")
        .fetch_one(&db)
        .await
        .unwrap();
    let game = |number: i16, map: &'static str, a_frags: i16, b_frags: i16| {
        sqlx::query(
            "insert into match_games (result_id, number, map_id, a_frags, b_frags)
             select $1, $2, id, $4, $5 from maps where slug = $3",
        )
        .bind(result)
        .bind(number)
        .bind(map)
        .bind(a_frags)
        .bind(b_frags)
        .execute(&db)
    };
    game(1, "awoken", 20, 10).await.unwrap();
    assert_eq!(code(game(2, "exile", 15, 15).await), CHECK); // empate
    assert_eq!(code(game(2, "exile", -1, 15).await), CHECK); // frags negativos
    assert_eq!(code(game(2, "awoken", 20, 10).await), UNIQUE); // mapa repetido

    let event = |kind: &'static str,
                 winner: Option<i64>,
                 maps: Option<(i16, i16)>,
                 comment: Option<&'static str>| {
        sqlx::query(
            "insert into match_events
                 (tenant_id, tournament_id, stage, match_key, kind, actor, winner, winner_maps,
                  loser_maps, comment)
             values ($1, $2, 1, 'wb-1-1', $3, $4, $5, $6, $7, $8)",
        )
        .bind(tenant)
        .bind(tournament)
        .bind(kind)
        .bind(p[0].id)
        .bind(winner)
        .bind(maps.map(|m| m.0))
        .bind(maps.map(|m| m.1))
        .bind(comment)
        .execute(&db)
    };
    event("reported", Some(a), Some((2, 0)), None)
        .await
        .unwrap();
    assert_eq!(code(event("contested", None, None, None).await), CHECK); // sem comentário
    assert_eq!(code(event("reported", Some(a), None, None).await), CHECK); // sem placar
    assert_eq!(code(event("confirmed", Some(a), None, None).await), CHECK); // placar onde não cabe
    assert_eq!(
        code(event("reported", Some(a), Some((1, 2)), None).await),
        CHECK
    ); // vencedor com menos
    assert_eq!(code(event("undone", None, None, None).await), CHECK);
}
