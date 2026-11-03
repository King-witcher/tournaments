//! As regras do banco para a sala de veto, gravando o estado inválido direto
//! por SQL.

use serde_json::Value;
use sqlx::PgPool;

use super::*;
use crate::schema::{CHECK, UNIQUE, count, exec, rejected, rule};

const TOKEN_A: &str = "2222222222222222222222";
const TOKEN_B: &str = "3333333333333333333333";

fn insert_room(team_size: i32, map_mode: &str, phase: bool, best_of: i32) -> String {
    format!(
        "insert into veto_rooms
             (manage_token, spectator_token, team_size, best_of, map_mode, veto_order,
              champion_phase, bans_per_series, mirror, same_team, step_seconds, team_pick_seconds,
              expires_at)
         values ('{TOKEN_A}', '{TOKEN_B}', {team_size}, {best_of}, '{map_mode}', 'bans_first',
                 {phase}, 1, true, false, 30, 60, now() + interval '1 hour')"
    )
}

/// Uma sala de duelo pela API, com os dois lados prontos.
async fn begun(db: &PgPool) -> (Router, Links, i64, Value) {
    let app = app(db.clone());
    let links = links_of(&app, room(1, 1)).await;
    let first = started(&app, &links).await;
    let id: i64 = sqlx::query_scalar("select id from veto_rooms")
        .fetch_one(db)
        .await
        .unwrap();
    (app, links, id, first)
}

#[tokio::test]
async fn room_settings_stay_in_their_ranges() {
    let (db, _lease) = crate::db::test_db().await;
    for (column, value, name) in [
        ("team_size", "5", "veto_rooms_team_size"),
        ("best_of", "2", "veto_rooms_best_of"),
        ("map_mode", "'vote'", "veto_rooms_map_mode"),
        ("veto_order", "'random'", "veto_rooms_veto_order"),
        ("bans_per_series", "4", "veto_rooms_bans"),
        ("no_repeat", "'match'", "veto_rooms_no_repeat"),
        ("step_seconds", "9", "veto_rooms_step_seconds"),
        ("team_pick_seconds", "14", "veto_rooms_team_pick_seconds"),
        ("manage_token", "'curto'", "veto_rooms_manage_token"),
        (
            "spectator_token",
            "'0OIl0OIl0OIl0OIl0OIl0O'",
            "veto_rooms_spectator_token",
        ),
    ] {
        let sql = format!(
            "insert into veto_rooms
                 (manage_token, spectator_token, team_size, best_of, map_mode, veto_order,
                  champion_phase, bans_per_series, mirror, same_team, step_seconds,
                  team_pick_seconds, expires_at)
             select {manage}, {spectator}, {team_size}, {best_of}, {map_mode}, {order}, true,
                    {bans}, true, false, {step}, {window}, now()",
            manage = if column == "manage_token" {
                value.to_owned()
            } else {
                format!("'{TOKEN_A}'")
            },
            spectator = if column == "spectator_token" {
                value.to_owned()
            } else {
                format!("'{TOKEN_B}'")
            },
            team_size = if column == "team_size" { value } else { "1" },
            best_of = if column == "best_of" { value } else { "1" },
            map_mode = if column == "map_mode" { value } else { "'off'" },
            order = if column == "veto_order" {
                value
            } else {
                "'bans_first'"
            },
            bans = if column == "bans_per_series" {
                value
            } else {
                "1"
            },
            step = if column == "step_seconds" {
                value
            } else {
                "30"
            },
            window = if column == "team_pick_seconds" {
                value
            } else {
                "60"
            },
        );
        let sql = if column == "no_repeat" {
            sql.replace("(manage_token,", "(no_repeat, manage_token,")
                .replace("select '", &format!("select {value}, '"))
        } else {
            sql
        };
        assert_eq!(
            rejected(exec(&db, &sql).await),
            rule(CHECK, name),
            "{column}"
        );
    }
    assert_eq!(
        rejected(exec(&db, &insert_room(1, "off", false, 1)).await),
        rule(CHECK, "veto_rooms_has_a_phase")
    );
    // A validade é obrigatória.
    let without = format!(
        "insert into veto_rooms
             (manage_token, spectator_token, team_size, best_of, map_mode, veto_order,
              champion_phase, bans_per_series, mirror, same_team, step_seconds, team_pick_seconds)
         values ('{TOKEN_A}', '{TOKEN_B}', 1, 1, 'off', 'bans_first', true, 1, true, false, 30, 60)"
    );
    assert_eq!(rejected(exec(&db, &without).await).0, "23502");
}

#[tokio::test]
async fn a_room_has_the_seats_of_its_mode_and_the_pool_of_its_map_mode() {
    let (db, _lease) = crate::db::test_db().await;
    let with_room = |room: String, rest: &'static str| {
        format!(
            "do $$ declare r bigint; begin
                 {room} returning id into r;
                 {rest}
             end $$"
        )
    };
    let seats = |side: &str, tokens: [&str; 2]| {
        format!(
            "insert into veto_room_seats (room_id, side, position, token) values
                 (r, '{side}', 1, '{a}'), (r, '{side}', 2, '{b}');",
            a = tokens[0].repeat(22),
            b = tokens[1].repeat(22),
        )
    };
    // Num `do`, o commit é o da própria instrução: a regra dispara ali.
    let two_a: &'static str = Box::leak(seats("a", ["4", "5"]).into_boxed_str());
    let sql = with_room(insert_room(2, "off", true, 1), two_a);
    assert_eq!(
        rejected(exec(&db, &sql).await),
        rule(CHECK, "veto_rooms_seats")
    );
    let four: &'static str =
        Box::leak(format!("{}{}", seats("a", ["4", "5"]), seats("b", ["6", "7"])).into_boxed_str());
    let sql = with_room(insert_room(2, "veto", true, 1), four);
    assert_eq!(
        rejected(exec(&db, &sql).await),
        rule(CHECK, "veto_rooms_pool"),
        "veto sem pool"
    );
    // Lugares a mais que o modo.
    let sql = with_room(insert_room(1, "off", true, 1), four);
    assert_eq!(
        rejected(exec(&db, &sql).await),
        rule(CHECK, "veto_rooms_seats")
    );
    // Assim, sim.
    exec(&db, &with_room(insert_room(2, "off", true, 1), four))
        .await
        .unwrap();
    // Um token por lugar, e um lugar por posição.
    assert_eq!(
        rejected(
            exec(
                &db,
                &format!(
                    "insert into veto_room_seats (room_id, side, position, token)
                     select id, 'a', 3, '{}' from veto_rooms",
                    "4".repeat(22)
                )
            )
            .await
        ),
        rule(UNIQUE, "veto_room_seats_token_key")
    );
    // Num pool, sem mapa fora de faixa nem repetido.
    assert_eq!(
        rejected(
            exec(
                &db,
                "insert into veto_room_maps (room_id, map_id, position)
                 select r.id, m.id, 33 from veto_rooms r, maps m limit 1"
            )
            .await
        ),
        rule(CHECK, "veto_room_maps_position")
    );
}

#[tokio::test]
async fn a_seat_name_is_set_once_and_keeps_its_size() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let links = links_of(&app, room(1, 1)).await;
    let set = |name: String| format!("update veto_room_seats set name = {name}");
    for name in ["''", "' a'", "'a '"] {
        assert_eq!(
            rejected(exec(&db, &set(name.to_owned())).await),
            rule(CHECK, "veto_room_seats_name"),
            "{name}"
        );
    }
    assert_eq!(
        rejected(exec(&db, &set(format!("'{}'", "x".repeat(33)))).await),
        rule(CHECK, "veto_room_seats_name")
    );
    exec(&db, &set(format!("'{}'", "é".repeat(32))))
        .await
        .unwrap();
    assert_eq!(
        rejected(exec(&db, &set("'outro'".to_owned())).await),
        rule(CHECK, "veto_room_seats_frozen")
    );
    assert_eq!(
        rejected(
            exec(
                &db,
                &format!(
                    "update veto_room_seats set token = '{}' where token = '{}'",
                    "9".repeat(22),
                    links.seats[0]
                )
            )
            .await
        ),
        rule(CHECK, "veto_room_seats_frozen")
    );
}

#[tokio::test]
async fn the_settings_do_not_change_but_the_validity_and_the_deadline_do() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    links_of(&app, room(1, 1)).await;
    for change in ["best_of = 3", "mirror = false", "step_seconds = 40"] {
        assert_eq!(
            rejected(exec(&db, &format!("update veto_rooms set {change}")).await),
            rule(CHECK, "veto_rooms_frozen"),
            "{change}"
        );
    }
    exec(
        &db,
        "update veto_rooms set expires_at = now() + interval '1 day', due_at = now()",
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn the_history_keeps_its_shape_and_only_grows() {
    let (db, _lease) = crate::db::test_db().await;
    let (_app, links, id, _) = begun(&db).await;
    let insert = |columns: &str, values: &str| {
        format!("insert into veto_room_events (room_id, kind, {columns}) values ({id}, {values})")
    };
    let seat: i64 = sqlx::query_scalar("select min(id) from veto_room_seats")
        .fetch_one(&db)
        .await
        .unwrap();
    let map: i64 = sqlx::query_scalar("select min(map_id) from veto_room_maps")
        .fetch_one(&db)
        .await
        .unwrap();
    let champion: i64 = sqlx::query_scalar("select min(id) from champions")
        .fetch_one(&db)
        .await
        .unwrap();
    let cases = [
        (
            insert("game, side", "'undo', 1, 'a'"),
            "veto_room_events_kind",
        ),
        (
            // Ban sem mapa nem campeão.
            insert("game, side", "'ban', 1, 'a'"),
            "veto_room_events_shape",
        ),
        (
            // Ban dos dois.
            insert(
                "game, side, map_id, champion_id",
                &format!("'ban', 1, 'a', {map}, {champion}"),
            ),
            "veto_room_events_shape",
        ),
        (
            // Pick de campeão sem a partida.
            insert(
                "game, side, champion_id",
                &format!("'pick', 1, 'a', {champion}"),
            ),
            "veto_room_events_pick_game",
        ),
        (
            // Janela em ban.
            insert(
                "game, side, champion_id, player_id",
                &format!("'ban', 1, 'a', {champion}, {seat}"),
            ),
            "veto_room_events_player",
        ),
        (
            // Regra cedida fora da janela.
            insert(
                "game, side, map_id, gave_way",
                &format!("'ban', 1, 'a', {map}, true"),
            ),
            "veto_room_events_gave_way",
        ),
        (
            // Sorteado com quem agiu.
            insert(
                "game, side, map_id, timeout, actor",
                &format!("'ban', 1, 'a', {map}, true, {seat}"),
            ),
            "veto_room_events_timeout",
        ),
        (
            // Pronto depois do começo.
            insert("side, actor", &format!("'ready', 'a', {seat}")),
            "veto_room_events_order",
        ),
    ];
    for (sql, name) in cases {
        let (code, got) = rejected(exec(&db, &sql).await);
        assert_eq!((code.as_str(), got.as_str()), (CHECK, name), "{sql}");
    }
    assert_eq!(
        rejected(exec(&db, "update veto_room_events set timeout = true").await),
        rule(CHECK, "veto_room_events_immutable")
    );
    // Numa sala que não começou: pronto sem o lugar, passo antes do começo.
    let other = links_of(&app(db.clone()), room(1, 1)).await;
    let other_room: i64 = sqlx::query_scalar("select max(id) from veto_rooms")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(
        rejected(
            exec(
                &db,
                &format!(
                    "insert into veto_room_events (room_id, kind, side)
                     values ({other_room}, 'ready', 'a')"
                )
            )
            .await
        ),
        rule(CHECK, "veto_room_events_shape")
    );
    let other_seat: i64 = sqlx::query_scalar("select id from veto_room_seats where token = $1")
        .bind(&other.seats[0])
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(
        rejected(
            exec(
                &db,
                &format!(
                    "insert into veto_room_events (room_id, kind, game, side, champion_id)
                     values ({other_room}, 'ban', 1, 'a', {champion})"
                )
            )
            .await
        ),
        rule(CHECK, "veto_room_events_order")
    );
    // Um pronto por lado.
    exec(
        &db,
        &format!(
            "insert into veto_room_events (room_id, kind, side, actor)
             values ({other_room}, 'ready', 'a', {other_seat})"
        ),
    )
    .await
    .unwrap();
    assert_eq!(
        rejected(
            exec(
                &db,
                &format!(
                    "insert into veto_room_events (room_id, kind, side, actor)
                     values ({other_room}, 'ready', 'a', {other_seat})"
                )
            )
            .await
        ),
        rule(UNIQUE, "veto_room_events_ready_once")
    );
    // O lugar de outra sala não vale.
    assert_eq!(
        rejected(
            exec(
                &db,
                &insert(
                    "game, side, map_id, actor",
                    &format!("'ban', 1, 'a', {map}, {other_seat}")
                )
            )
            .await
        )
        .0,
        crate::schema::FOREIGN_KEY
    );
    assert!(!links.manage.is_empty());
}

#[tokio::test]
async fn deleting_the_room_leaves_nothing_and_an_account_leaves_the_rooms() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _, _, _) = begun(&db).await;
    let ana = user(&db, "Ana").await;
    let (status, _) = create_room(&app, Some(&ana), room(1, 1)).await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    exec(&db, &format!("delete from users where id = {}", ana.id))
        .await
        .unwrap();
    assert_eq!(
        count(
            &db,
            "select count(*) from veto_rooms where created_by is null"
        )
        .await,
        2
    );
    exec(&db, "delete from veto_rooms").await.unwrap();
    for table in ["veto_room_seats", "veto_room_events", "veto_room_maps"] {
        assert_eq!(
            count(&db, &format!("select count(*) from {table}")).await,
            0
        );
    }
}
