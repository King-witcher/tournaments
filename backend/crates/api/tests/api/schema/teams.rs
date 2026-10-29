//! Times: no duelo, o time de um jogador nasce com a inscrição, tem só ele
//! (que é o capitão) e nenhum nome, e some com a inscrição; o capitão é
//! sempre do time; cada mapa grava quem jogou por lado (a escalação), que não
//! muda e tem alguém dos dois lados.

use super::*;
use crate::common::*;
use crate::matches::{copa, report, single, start};

/// `(team_id, captain)` do inscrito `user` no torneio `slug`.
async fn team_of(db: &PgPool, slug: &str, user: &User) -> (i64, i64) {
    sqlx::query_as(
        "select tm.id, tm.captain from participants p
         join teams tm on tm.tournament_id = p.tournament_id and tm.id = p.team_id
         join tournaments t on t.id = p.tournament_id
         where t.slug = $1 and p.user_id = $2",
    )
    .bind(slug)
    .bind(user.id)
    .fetch_one(db)
    .await
    .unwrap()
}

#[tokio::test]
async fn a_duel_registration_is_a_team_of_one() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&ana, &bia]).await;

    // Cada inscrição nasce com o próprio time, de capitão ela mesma.
    let (ana_team, captain) = team_of(&db, "copa", &ana).await;
    assert_eq!(captain, ana.id);
    let (bia_team, _) = team_of(&db, "copa", &bia).await;
    assert_ne!(ana_team, bia_team);

    // Time com nome, ou com mais de um jogador, não é de duelo.
    assert_eq!(
        rejected(
            exec(
                &db,
                &format!("update teams set name = 'Os Bons' where id = {ana_team}")
            )
            .await
        ),
        rule(CHECK, "teams_duel_solo")
    );
    // A Bia no time da Ana: o time dela fica sem o capitão.
    let move_bia = format!(
        "update participants set team_id = {ana_team} where user_id = {}",
        bia.id
    );
    assert_eq!(
        rejected(exec(&db, &move_bia).await),
        rule(CHECK, "teams_captain_member")
    );
    // Inscrito do duelo sempre tem time.
    let no_team = format!(
        "update participants set team_id = null where user_id = {}",
        ana.id
    );
    assert_eq!(
        rejected(exec(&db, &no_team).await),
        rule(CHECK, "participants_duel_team")
    );
    // O capitão é do time (e capitão de um time só).
    let captain = format!(
        "update teams set captain = {} where id = {ana_team}",
        bia.id
    );
    assert_eq!(
        rejected(exec(&db, &captain).await),
        rule(UNIQUE, "teams_captain_key")
    );
    // O capitão é um inscrito do torneio.
    let zeca = member(&app, &db, &ana, "Zeca").await;
    let outsider = format!(
        "update teams set captain = {} where id = {ana_team}",
        zeca.id
    );
    assert_eq!(
        rejected(exec(&db, &outsider).await),
        rule(FOREIGN_KEY, "teams_captain_fkey")
    );

    // A inscrição sai e leva o time.
    let (status, body) = delete(
        &app,
        "/tenants/quake-br/tournaments/copa/registration",
        &bia,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::NO_CONTENT, "{body}");
    assert_eq!(
        count(
            &db,
            &format!("select count(*) from teams where id = {bia_team}")
        )
        .await,
        0
    );
}

#[tokio::test]
async fn each_map_keeps_who_played_it() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    start(&app, &ana, copa(single(3, 3)), &[&ana, &bia]).await;
    let (status, body) = report(&app, &bia, "wb-1-1", 1, crate::matches::game("b", "awoken")).await;
    assert_eq!(status, axum::http::StatusCode::OK, "{body}");

    // A escalação nasce com o mapa: o jogador de cada lado.
    let lineup: Vec<(String, i64)> =
        sqlx::query_as("select side, user_id from match_game_players order by side")
            .fetch_all(&db)
            .await
            .unwrap();
    assert_eq!(lineup, [("a".into(), ana.id), ("b".into(), bia.id)]);
    let game = count(&db, "select id from match_games").await;

    // Não muda, e só entra quem é do time daquele lado.
    assert_eq!(
        rejected(exec(&db, "update match_game_players set side = 'a'").await),
        rule(CHECK, "match_game_players_immutable")
    );
    let (tenant, tournament): (i64, i64) = sqlx::query_as("select tenant_id, id from tournaments")
        .fetch_one(&db)
        .await
        .unwrap();
    // Sem ninguém de um lado, o commit recusa.
    let drop_bia = format!("delete from match_game_players where user_id = {}", bia.id);
    assert_eq!(
        rejected(exec(&db, &drop_bia).await),
        rule(CHECK, "match_game_players_required")
    );
    // Quem joga um lado é do time daquele lado.
    let wrong_side = format!(
        "insert into match_game_players (tenant_id, tournament_id, game_id, side, user_id)
         values ({tenant}, {tournament}, {game}, 'a', {})",
        bia.id
    );
    let mut tx = db.begin().await.unwrap();
    sqlx::query(sqlx::AssertSqlSafe(drop_bia))
        .execute(&mut *tx)
        .await
        .unwrap();
    assert_eq!(
        rejected(
            sqlx::query(sqlx::AssertSqlSafe(wrong_side))
                .execute(&mut *tx)
                .await
        ),
        rule(CHECK, "match_game_players_team")
    );
}
