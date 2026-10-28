//! A conversão das séries enviadas inteiras para mapas (migration
//! `20261101010000_games`) e a janela padrão de 10 minutos
//! (`20261101000000_contest_window`): o banco vai até a migration de antes,
//! recebe dados na forma antiga e então aplica as novas.

use serde_json::{Value, json};
use sqlx::PgPool;

use super::*;
use crate::common::*;

/// A última migration antes do envio mapa a mapa.
const BEFORE: i64 = 20261031010000;

/// As rotas de hoje gravam no outbox, que só existe a partir da migration
/// `20261104000000_realtime`: no schema antigo, uma tabela que aceita as mesmas
/// gravações, que sai antes de as migrations novas criarem a de verdade.
async fn outbox_stand_in(db: &PgPool, create: bool) {
    let sql = if create {
        "create table outbox (
             id bigint generated always as identity,
             xact xid8 not null default pg_current_xact_id(),
             event jsonb not null
         );
         create unique index on outbox (xact, md5(event::text))"
    } else {
        "drop table outbox"
    };
    sqlx::raw_sql(sql).execute(db).await.unwrap();
}

#[sqlx::test(migrations = false)]
async fn series_sent_whole_become_maps(db: PgPool) {
    api::MIGRATOR.run_to(BEFORE, &db).await.unwrap();
    outbox_stand_in(&db, true).await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    start(
        &app,
        &ana,
        copa(single(3, 3)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    for slug in ["rascunho", "quinze"] {
        create_tournament(&app, &ana, duel(slug)).await;
    }
    sqlx::query(
        "update tournaments set contest_minutes = case slug when 'quinze' then 15 else 5 end",
    )
    .execute(&db)
    .await
    .unwrap();

    let t = sqlx::query!("select tenant_id, id from tournaments where slug = 'copa'")
        .fetch_one(&db)
        .await
        .unwrap();
    let seeded: Vec<i64> = sqlx::query_scalar!(
        "select id from participants where tournament_id = $1 and seed is not null order by seed",
        t.id
    )
    .fetch_all(&db)
    .await
    .unwrap();
    let (p1, p2, p3, p4) = (seeded[0], seeded[1], seeded[2], seeded[3]);
    // wb-1-1: P1 2 x 1 P4, enviado por P4 com detail e confirmado por P1.
    // wb-1-2: P3 venceu P2 2 x 0, lançado pela organização, sem detalhe.
    // wb-2-1: P1 2 x 1 P3, enviado por P3 e ainda pendente.
    sqlx::query(
        r#"insert into match_results
               (tenant_id, tournament_id, stage, match_key, best_of, side_a, side_b, status,
                winner, a_wins, b_wins, reported_side, reported_by, confirm_by, confirmed_via,
                confirmed_by, confirmed_at, position)
           values
               ($1, $2, 1, 'wb-1-1', 3, $3, $6, 'confirmed', $3, 2, 1, 'b', $7,
                now() + interval '5 minutes', 'opponent', $8, now(), 1),
               ($1, $2, 1, 'wb-1-2', 3, $4, $5, 'confirmed', $5, 0, 2, null, $9, null,
                'organizer', $9, now(), 2),
               ($1, $2, 1, 'wb-2-1', 3, $3, $5, 'pending', $3, 2, 1, 'b', $10,
                now() + interval '5 minutes', null, null, null, null)"#,
    )
    .bind(t.tenant_id)
    .bind(t.id)
    .bind(p1)
    .bind(p2)
    .bind(p3)
    .bind(p4)
    .bind(p[3].id)
    .bind(p[0].id)
    .bind(ana.id)
    .bind(p[2].id)
    .execute(&db)
    .await
    .unwrap();
    sqlx::query(
        r#"insert into match_games (result_id, number, map_id, a_frags, b_frags)
           select r.id, g.number, m.id, g.a, g.b
           from match_results r,
                (values (1, 'awoken', 20, 10), (2, 'exile', 5, 10), (3, 'insomnia', 30, 2))
                    as g(number, slug, a, b)
           join maps m on m.slug = g.slug
           where r.match_key = 'wb-1-1'"#,
    )
    .execute(&db)
    .await
    .unwrap();

    outbox_stand_in(&db, false).await;
    api::MIGRATOR.run(&db).await.unwrap();

    let maps = sqlx::query!(
        r#"select g.match_key, g.number, g.winner, m.slug as "slug?", g.a_frags, g.status,
                  g.reported_side, g.confirmed_via
           from match_games g left join maps m on m.id = g.map_id
           order by g.match_key, g.number"#
    )
    .fetch_all(&db)
    .await
    .unwrap();
    let rows: Vec<Value> = maps
        .iter()
        .map(|g| {
            json!([
                g.match_key,
                g.number,
                g.winner,
                g.slug,
                g.a_frags,
                g.status,
                g.reported_side,
                g.confirmed_via
            ])
        })
        .collect();
    let c = "confirmed";
    assert_eq!(
        rows,
        [
            json!(["wb-1-1", 1, p1, "awoken", 20, c, "b", "opponent"]),
            json!(["wb-1-1", 2, p4, "exile", 5, c, "b", "opponent"]),
            json!(["wb-1-1", 3, p1, "insomnia", 30, c, "b", "opponent"]),
            // Sem detalhe: sem mapa, os do perdedor primeiro (aqui, nenhum).
            json!(["wb-1-2", 1, p3, null, null, c, null, "organizer"]),
            json!(["wb-1-2", 2, p3, null, null, c, null, "organizer"]),
            json!(["wb-2-1", 1, p3, null, null, "pending", "b", null]),
            json!(["wb-2-1", 2, p1, null, null, "pending", "b", null]),
            json!(["wb-2-1", 3, p1, null, null, "pending", "b", null]),
        ]
    );
    // A série pendente sai de match_results: ainda não estava decidida.
    let decided: Vec<String> =
        sqlx::query_scalar!("select match_key from match_results order by position")
            .fetch_all(&db)
            .await
            .unwrap();
    assert_eq!(decided, ["wb-1-1", "wb-1-2"]);
    // Janela: 5, o padrão antigo, vira 10 onde o torneio não começou.
    let windows: Vec<(String, i32)> =
        sqlx::query!("select slug, contest_minutes from tournaments order by slug")
            .fetch_all(&db)
            .await
            .unwrap()
            .into_iter()
            .map(|r| (r.slug, r.contest_minutes))
            .collect();
    assert_eq!(
        windows,
        [
            ("copa".to_owned(), 5),
            ("quinze".to_owned(), 15),
            ("rascunho".to_owned(), 10)
        ]
    );

    // A chave sai igual, e a final fecha quando os mapas confirmam.
    let b = bracket(&app, &ana).await;
    let final_ = find(&b, "wb-2-1");
    assert_eq!(
        (&final_["state"], &final_["pending_maps"], &sides(final_)),
        (
            &json!("ready"),
            &json!(3),
            &(json!(p[0].id), json!(p[2].id))
        )
    );
    let d = detail(&app, &ana, "wb-1-1").await;
    assert_eq!(d["games"][1]["map"]["name"], "Exile");
    assert_eq!(d["summary"]["result"]["confirmed_via"], "opponent");
    sqlx::query("update match_games set reported_at = reported_at - interval '1 hour', confirm_by = confirm_by - interval '1 hour' where status = 'pending'")
        .execute(&db)
        .await
        .unwrap();
    let b = bracket(&app, &ana).await;
    assert_eq!(
        (&find(&b, "wb-2-1")["result"]["a_wins"], &b["status"]),
        (&json!(2), &json!("finished"))
    );
}
