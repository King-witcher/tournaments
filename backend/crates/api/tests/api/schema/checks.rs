//! Coerência e forma: estado, JSON, identidade imutável, vagas, autoria e
//! convite dentro do tenant e do torneio, formato de logins e segredos.

use serde_json::json;
use sqlx::PgPool;

use super::*;
use crate::common::*;

async fn tenant_id(db: &PgPool, slug: &str) -> i64 {
    sqlx::query_scalar("select id from tenants where slug = $1")
        .bind(slug)
        .fetch_one(db)
        .await
        .unwrap()
}

async fn tournament_id(db: &PgPool, slug: &str) -> i64 {
    sqlx::query_scalar("select id from tournaments where slug = $1")
        .bind(slug)
        .fetch_one(db)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn tournaments_keep_state_and_shape(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;

    // Rascunho pode ficar sem data; publicado, não.
    exec(&db, "update tournaments set starts_at = null")
        .await
        .unwrap();
    assert_eq!(
        rejected(exec(&db, "update tournaments set status = 'registration'").await),
        rule(CHECK, "tournaments_starts_at_required")
    );
    exec(&db, "update tournaments set starts_at = now()")
        .await
        .unwrap();
    // Check-in sem janela não existe.
    assert_eq!(
        rejected(
            exec(
                &db,
                "update tournaments set status = 'check_in', check_in_minutes = null"
            )
            .await
        ),
        rule(CHECK, "tournaments_check_in_window")
    );

    // O JSON tem a forma do engine.
    for rules in [
        json!({ "bans_per_game": 4, "no_repeat": null }),
        json!({ "bans_per_game": 1, "no_repeat": { "who": "player", "scope": "season" } }),
        json!({ "bans_per_game": 1 }),
        json!([]),
    ] {
        let result = sqlx::query("update tournaments set champion_rules = $1")
            .bind(&rules)
            .execute(&db)
            .await;
        assert_eq!(
            rejected(result),
            rule(CHECK, "tournaments_champion_rules_shape"),
            "{rules}"
        );
    }
    let single = |series: serde_json::Value| json!({ "format": "elimination", "kind": { "kind": "single" }, "series": series });
    for config in [
        single(json!({ "default": 2, "finals": 3 })),
        single(json!({ "default": 3 })),
        json!({ "format": "elimination", "kind": { "kind": "double" },
                "series": { "default": 3, "finals": 5 } }),
        json!({ "format": "swiss" }),
    ] {
        let result = sqlx::query("update tournament_stages set config = $1")
            .bind(&config)
            .execute(&db)
            .await;
        assert_eq!(
            rejected(result),
            rule(CHECK, "tournament_stages_config_shape"),
            "{config}"
        );
    }
    sqlx::query("update tournament_stages set config = $1")
        .bind(single(json!({ "default": 1, "finals": 3 })))
        .execute(&db)
        .await
        .unwrap();

    // Tenant, slug e idioma original não mudam.
    for change in ["slug = 'outra'", "locale = 'es'"] {
        let sql = format!("update tournaments set {change}");
        assert_eq!(
            rejected(exec(&db, &sql).await),
            rule(CHECK, "tournaments_identity_immutable"),
            "{change}"
        );
    }
    assert_eq!(
        rejected(exec(&db, "update tenants set slug = 'outro'").await),
        rule(CHECK, "tenants_slug_immutable")
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn every_tournament_has_a_stage(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let no_stage = rule(CHECK, "tournament_stages_required");

    // Trocar as fases na mesma transação (como o PATCH) pode.
    let mut tx = db.begin().await.unwrap();
    sqlx::query("delete from tournament_stages")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query(
        "insert into tournament_stages (tournament_id, position, config)
         select id, 1, '{\"format\": \"elimination\", \"kind\": {\"kind\": \"single\"},
                         \"series\": {\"default\": 3, \"finals\": 3}}'
         from tournaments",
    )
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();

    // Ficar sem nenhuma, não.
    let mut tx = db.begin().await.unwrap();
    sqlx::query("delete from tournament_stages")
        .execute(&mut *tx)
        .await
        .unwrap();
    assert_eq!(rejected(tx.commit().await), no_stage);

    // Nem nascer sem.
    let tenant = tenant_id(&db, "quake-br").await;
    let mut tx = db.begin().await.unwrap();
    sqlx::query(
        "insert into tournaments (tenant_id, slug, locale, name, mode, max_participants, champion_rules)
         values ($1, 'outra', 'pt', 'Outra', 'duel', 8, '{\"bans_per_game\": 1, \"no_repeat\": null}')",
    )
    .bind(tenant)
    .execute(&mut *tx)
    .await
    .unwrap();
    assert_eq!(rejected(tx.commit().await), no_stage);

    // Excluir o torneio leva as fases sem esbarrar na regra.
    exec(&db, "delete from tournaments").await.unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn translations_are_never_in_the_original_locale(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let tournament = tournament_id(&db, "copa").await;
    let translate = |locale: &'static str| {
        sqlx::query(
            "insert into tournament_translations (tournament_id, locale, name) values ($1, $2, 'Copa')",
        )
        .bind(tournament)
        .bind(locale)
        .execute(&db)
    };
    let original = rule(CHECK, "tournament_translations_not_original");

    assert_eq!(rejected(translate("pt").await), original);
    translate("es").await.unwrap();
    assert_eq!(
        rejected(exec(&db, "update tournament_translations set locale = 'pt'").await),
        original
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn participants_fit_and_stay_inside_tenant_and_tournament(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let cleo = member(&app, &db, &ana, "Cleo").await;
    let dani = member(&app, &db, &ana, "Dani").await;
    create_tournament(
        &app,
        &ana,
        with(duel("copa"), json!({ "max_participants": 3 })),
    )
    .await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&ana, &bia, &cleo]).await;
    let tenant = tenant_id(&db, "quake-br").await;
    let copa = tournament_id(&db, "copa").await;

    // Lotado: ninguém mais entra, e as vagas não descem abaixo dos inscritos.
    let insert = format!(
        "insert into participants (tenant_id, tournament_id, user_id) values ({tenant}, {copa}, {})",
        dani.id
    );
    assert_eq!(
        rejected(exec(&db, &insert).await),
        rule(CHECK, "participants_capacity")
    );
    assert_eq!(
        rejected(exec(&db, "update tournaments set max_participants = 2").await),
        rule(CHECK, "tournaments_max_participants_below_registered")
    );

    // Autoria só de membro do mesmo tenant.
    let zeca = user(&db, "Zeca").await;
    create(&app, &zeca, "outro").await;
    let by_outsider = |column: &str| {
        format!(
            "update participants set checked_in_at = now(), {column} = {} where user_id = {}",
            zeca.id, bia.id
        )
    };
    assert_eq!(
        rejected(exec(&db, &by_outsider("checked_in_by")).await),
        rule(FOREIGN_KEY, "participants_checked_in_by_fkey")
    );
    assert_eq!(
        rejected(exec(&db, &by_outsider("added_by")).await),
        rule(FOREIGN_KEY, "participants_added_by_fkey")
    );
    let sql = format!(
        "update participants set checked_in_by = {} where user_id = {}",
        ana.id, bia.id
    );
    assert_eq!(
        rejected(exec(&db, &sql).await),
        rule(CHECK, "participants_check")
    );
    let sql = format!(
        "insert into tenant_invites (tenant_id, token_hash, created_by, expires_at)
         values ({tenant}, sha256('x'), {}, now() + interval '1 day')",
        zeca.id
    );
    assert_eq!(
        rejected(exec(&db, &sql).await),
        rule(FOREIGN_KEY, "tenant_invites_created_by_fkey")
    );

    // O convite da inscrição é um link deste torneio.
    create_tournament(&app, &ana, duel("outra")).await;
    for slug in ["copa", "outra"] {
        let uri = format!("/tenants/quake-br/tournaments/{slug}/invites");
        let (status, body) = post_empty(&app, &uri, &ana).await;
        assert_eq!(status, axum::http::StatusCode::CREATED, "{body}");
    }
    let link = |slug: &'static str| {
        format!(
            "update participants set invite_id = (
                 select i.id from tenant_invites i join tournaments t on t.id = i.tournament_id
                 where t.slug = '{slug}')
             where user_id = {}",
            cleo.id
        )
    };
    exec(&db, &link("copa")).await.unwrap();
    assert_eq!(
        rejected(exec(&db, &link("outra")).await),
        rule(FOREIGN_KEY, "participants_invite_id_fkey")
    );
    let tenant_invite = format!(
        "update participants set invite_id = (
             select id from tenant_invites where tournament_id is null limit 1)
         where user_id = {}",
        cleo.id
    );
    assert_eq!(
        rejected(exec(&db, &tenant_invite).await),
        rule(FOREIGN_KEY, "participants_invite_id_fkey")
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn logins_and_secrets_keep_their_format(db: PgPool) {
    let ana = user(&db, "Ana").await;
    let password = |username: &'static str, hash: &'static str| {
        sqlx::query(
            "insert into password_credentials (user_id, username, password_hash)
             values ($1, $2, $3)",
        )
        .bind(ana.id)
        .bind(username)
        .bind(hash)
        .execute(&db)
    };
    let identity = |provider: &'static str, id: &'static str| {
        sqlx::query(
            "insert into user_identities (user_id, provider, provider_user_id) values ($1, $2, $3)",
        )
        .bind(ana.id)
        .bind(provider)
        .bind(id)
        .execute(&db)
    };
    let argon = "$argon2id$v=19$m=19456,t=2,p=1$c2FsdA$aGFzaA";

    for username in ["ab", "com espaço", "ação", "a23456789012345678901"] {
        assert_eq!(
            rejected(password(username, argon).await),
            rule(CHECK, "password_credentials_username_format"),
            "{username}"
        );
    }
    assert_eq!(
        rejected(password("Ana_1", "hunter22").await),
        rule(CHECK, "password_credentials_password_hash_format")
    );
    assert_eq!(
        rejected(identity("discord", "").await),
        rule(CHECK, "user_identities_provider_user_id_check")
    );
    // A senha saiu de user_identities, que só tem provedores externos.
    assert_eq!(
        rejected(identity("password", "Ana_1").await),
        rule(CHECK, "user_identities_provider_check")
    );
    password("Ana_1", argon).await.unwrap();
    let bia = user(&db, "Bia").await;
    assert_eq!(
        rejected(
            sqlx::query(
                "insert into password_credentials (user_id, username, password_hash)
                 values ($1, 'ANA_1', $2)"
            )
            .bind(bia.id)
            .bind(argon)
            .execute(&db)
            .await
        ),
        rule(UNIQUE, "password_credentials_username_idx")
    );

    for (change, name) in [
        ("display_name = ''", "users_display_name_check"),
        (
            "avatar_url = 'javascript:alert(1)'",
            "users_avatar_url_https",
        ),
        ("email = 'Ana@outlook.com'", "users_email_format"),
        ("email = 'ana+x@outlook.com'", "users_email_format"),
        ("email = 'ana.silva@gmail.com'", "users_email_format"),
        ("email = 'anasilva@googlemail.com'", "users_email_format"),
        ("email = 'ana@localhost'", "users_email_format"),
    ] {
        let sql = format!("update users set {change}");
        assert_eq!(
            rejected(exec(&db, &sql).await),
            rule(CHECK, name),
            "{change}"
        );
    }
    let email = |id: i64| format!("update users set email = 'ana@outlook.com' where id = {id}");
    exec(&db, &email(ana.id)).await.unwrap();
    assert_eq!(
        rejected(exec(&db, &email(bia.id)).await),
        rule(UNIQUE, "users_email_key")
    );
    assert_eq!(
        rejected(
            exec(
                &db,
                "insert into champions (slug, name) values ('anarki-2', 'Anarki')"
            )
            .await
        ),
        rule(UNIQUE, "champions_name_key")
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn invite_tokens_are_hashes(db: PgPool) {
    let (_, ana, _) = quake_br(&db).await;
    let tenant = tenant_id(&db, "quake-br").await;
    let result = sqlx::query(
        "insert into tenant_invites (tenant_id, token_hash, created_by, expires_at)
         values ($1, $2, $3, now() + interval '1 day')",
    )
    .bind(tenant)
    .bind(&[7u8; 16][..])
    .bind(ana.id)
    .execute(&db)
    .await;
    assert_eq!(
        rejected(result),
        rule(CHECK, "tenant_invites_token_hash_sha256")
    );
}
