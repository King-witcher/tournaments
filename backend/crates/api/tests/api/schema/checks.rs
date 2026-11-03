//! Coerência e forma: estado, JSON, identidade imutável, vagas, autoria dentro
//! do tenant e do torneio, formato de logins e segredos, o token do link de
//! convite de cada escopo, senha só com email.

use serde_json::json;
use sqlx::PgPool;

use super::*;
use crate::common::*;

pub(super) async fn tenant_id(db: &PgPool, slug: &str) -> i64 {
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

#[tokio::test]
async fn tournaments_keep_state_and_shape() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;

    // A data de início é opcional em qualquer status.
    exec(
        &db,
        "update tournaments set starts_at = null, status = 'registration'",
    )
    .await
    .unwrap();
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
        json!({ "bans_per_series": 4, "mirror": true, "same_team": false, "no_repeat": null }),
        json!({ "bans_per_series": 1, "mirror": true, "same_team": false, "no_repeat": { "who": "player", "scope": "season" } }),
        json!({ "bans_per_series": 1, "mirror": true, "same_team": false, "no_repeat": { "who": "player", "scope": "match" } }),
        json!({ "bans_per_series": 1, "mirror": "yes", "same_team": false, "no_repeat": null }),
        json!({ "bans_per_series": 1 }),
        json!({ "bans_per_game": 1, "no_repeat": null, "team_duplicates": false }),
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
        json!({ "format": "swiss", "best_of": 3, "rounds": 0 }),
        json!({ "format": "swiss", "best_of": 3, "rounds": 128 }),
        json!({ "format": "swiss", "best_of": 3, "rounds": "3" }),
        json!({ "format": "round_robin", "best_of": 2 }),
        json!({ "format": "round_robin" }),
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
    for config in [
        json!({ "format": "round_robin", "best_of": 3 }),
        json!({ "format": "swiss", "best_of": 1, "rounds": null }),
        json!({ "format": "swiss", "best_of": 5, "rounds": 127 }),
    ] {
        sqlx::query("update tournament_stages set config = $1")
            .bind(&config)
            .execute(&db)
            .await
            .unwrap();
    }
    sqlx::query("update tournament_stages set config = $1")
        .bind(single(json!({ "default": 1, "finals": 3 })))
        .execute(&db)
        .await
        .unwrap();

    // O idioma original não muda.
    assert_eq!(
        rejected(exec(&db, "update tournaments set locale = 'es'").await),
        rule(CHECK, "tournaments_identity_immutable"),
    );
    // O slug do torneio muda só no rascunho (este está publicado); o da
    // comunidade, sempre.
    assert_eq!(
        rejected(exec(&db, "update tournaments set slug = 'outra'").await),
        rule(CHECK, "tournaments_slug_after_draft"),
    );
    exec(&db, "update tenants set slug = 'outro'")
        .await
        .unwrap();
}

#[tokio::test]
async fn tenant_description_is_null_or_within_the_limit() {
    let (db, _lease) = crate::db::test_db().await;
    let _ = quake_br(&db).await;

    for value in ["''", "repeat('é', 2001)"] {
        let sql = format!("update tenants set description = {value}");
        assert_eq!(
            rejected(exec(&db, &sql).await),
            rule(CHECK, "tenants_description"),
            "{value}"
        );
    }
    for value in ["null", "repeat('é', 2000)"] {
        let sql = format!("update tenants set description = {value}");
        assert_eq!(exec(&db, &sql).await.unwrap(), 1, "{value}");
    }
}

#[tokio::test]
async fn every_tournament_has_a_stage() {
    let (db, _lease) = crate::db::test_db().await;
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
        "insert into tournaments (tenant_id, slug, locale, name, mode, max_participants, champion_rules, invite_token)
         values ($1, 'outra', 'pt', 'Outra', 'duel', 8, '{\"bans_per_series\": 1, \"mirror\": true, \"same_team\": false, \"no_repeat\": null}', $2)",
    )
    .bind(tenant)
    .bind(INVITE_TOKEN)
    .execute(&mut *tx)
    .await
    .unwrap();
    assert_eq!(rejected(tx.commit().await), no_stage);

    // Excluir o torneio leva as fases sem esbarrar na regra.
    exec(&db, "delete from tournaments").await.unwrap();
}

#[tokio::test]
async fn translations_are_never_in_the_original_locale() {
    let (db, _lease) = crate::db::test_db().await;
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

#[tokio::test]
async fn participants_fit_and_stay_inside_tenant_and_tournament() {
    let (db, _lease) = crate::db::test_db().await;
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
}

#[tokio::test]
async fn logins_and_secrets_keep_their_format() {
    let (db, _lease) = crate::db::test_db().await;
    let ana = user(&db, "Ana").await;
    let email =
        |id: i64, address: &str| format!("update users set email = '{address}' where id = {id}");
    // A senha pede o email da conta (`password_needs_the_account_email`).
    exec(&db, &email(ana.id, "ana@outlook.com")).await.unwrap();
    let password = |hash: &'static str| {
        sqlx::query("insert into password_credentials (user_id, password_hash) values ($1, $2)")
            .bind(ana.id)
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

    assert_eq!(
        rejected(password("hunter22").await),
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
    // O `sub` do Google: até 255 caracteres ASCII visíveis.
    let long_sub: &'static str = "1".repeat(256).leak();
    for sub in ["com espaço", "ação", long_sub] {
        assert_eq!(
            rejected(identity("google", sub).await),
            rule(CHECK, "user_identities_google_sub_format"),
            "{sub}"
        );
    }
    identity("google", "110248495921238986420").await.unwrap();
    password(argon).await.unwrap();
    // Uma senha por conta.
    assert_eq!(
        rejected(password(argon).await),
        rule(UNIQUE, "password_credentials_pkey")
    );
    let bia = user(&db, "Bia").await;
    exec(&db, &email(bia.id, "bia@outlook.com")).await.unwrap();

    for (change, name) in [
        ("display_name = ''", "users_display_name_check"),
        (
            "avatar_url = 'javascript:alert(1)'",
            "users_avatar_url_https",
        ),
        ("email = 'Ana@outlook.com'", "users_email_format"),
        ("email = 'ana+@outlook.com'", "users_email_format"),
        ("email = 'ana+x y@outlook.com'", "users_email_format"),
        ("email = '+x@outlook.com'", "users_email_format"),
        ("email = 'ana@localhost'", "users_email_format"),
    ] {
        let sql = format!("update users set {change}");
        assert_eq!(
            rejected(exec(&db, &sql).await),
            rule(CHECK, name),
            "{change}"
        );
    }
    // Sem EMAIL_STRICT (dev e staging), a API grava as variações como vieram.
    for address in [
        "bia+x@outlook.com",
        "bia+a.b+c@outlook.com",
        "bia.silva@gmail.com",
        "biasilva@googlemail.com",
    ] {
        exec(&db, &email(bia.id, address)).await.unwrap();
    }
    assert_eq!(
        rejected(exec(&db, &email(bia.id, "ana@outlook.com")).await),
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

#[tokio::test]
async fn password_needs_the_account_email() {
    let (db, _lease) = crate::db::test_db().await;
    let ana = user(&db, "Ana").await;
    let bia = user(&db, "Bia").await;
    let password = format!(
        "insert into password_credentials (user_id, password_hash)
         values ({}, '$argon2id$v=19$m=19456,t=2,p=1$c2FsdA$aGFzaA')",
        ana.id
    );
    let email = |value: &str| format!("update users set email = {value} where id = {}", ana.id);
    let required = rule(CHECK, "password_credentials_email_required");

    assert_eq!(rejected(exec(&db, &password).await), required);
    exec(&db, &email("'ana@outlook.com'")).await.unwrap();
    exec(&db, &password).await.unwrap();
    assert_eq!(rejected(exec(&db, &email("null")).await), required);
    // Trocar de email pode; passar a senha para quem não tem email, não.
    exec(&db, &email("'ana@gmail.com'")).await.unwrap();
    let to_bia = format!(
        "update password_credentials set user_id = {} where user_id = {}",
        bia.id, ana.id
    );
    assert_eq!(rejected(exec(&db, &to_bia).await), required);

    // Sem a senha, o email sai.
    let remove = format!(
        "delete from password_credentials where user_id = {}",
        ana.id
    );
    exec(&db, &remove).await.unwrap();
    exec(&db, &email("null")).await.unwrap();
}

#[tokio::test]
async fn every_scope_has_one_well_formed_invite_token() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    create(&app, &ana, "outra").await;
    create_tournament(&app, &ana, duel("copa")).await;
    create_tournament(&app, &ana, duel("taca")).await;

    let set = |table: &str, value: &str, slug: &str| {
        format!("update {table} set invite_token = {value} where slug = '{slug}'")
    };
    for (table, one, other) in [
        ("tenants", "quake-br", "outra"),
        ("tournaments", "copa", "taca"),
    ] {
        // Sempre um.
        assert_eq!(
            rejected(exec(&db, &set(table, "null", one)).await),
            rule(NOT_NULL, "")
        );
        // Base58 de 22 caracteres: sem `0`, `O`, `I` e `l`, sem tamanho curto.
        for bad in [
            "''",
            "'abc'",
            "'0Fq2xKpR9mWbTnVc3hJdYe'",
            "'7Fq2xKpR9mWbTnVc3hJdY-'",
        ] {
            assert_eq!(
                rejected(exec(&db, &set(table, bad, one)).await),
                rule(CHECK, &format!("{table}_invite_token_format")),
                "{bad}"
            );
        }
        // Nunca repetido entre comunidades, nem entre torneios.
        let literal = format!("'{INVITE_TOKEN}'");
        exec(&db, &set(table, &literal, one)).await.unwrap();
        assert_eq!(
            rejected(exec(&db, &set(table, &literal, other)).await),
            rule(UNIQUE, &format!("{table}_invite_token_key"))
        );
    }
}

#[tokio::test]
async fn admission_and_check_in_on_entry() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    let tenant = tenant_id(&db, "quake-br").await;
    let copa = tournament_id(&db, "copa").await;

    // O tipo de inscrição só tem dois valores.
    assert_eq!(
        rejected(exec(&db, "update tournaments set admission = 'private'").await),
        rule(CHECK, "tournaments_admission_check")
    );

    // Durante o check-in, quem entra já tem o check-in feito; fora dele, não.
    let insert = |check_in: &str| {
        format!(
            "insert into participants (tenant_id, tournament_id, user_id, checked_in_at)
             values ({tenant}, {copa}, {}, {check_in})",
            bia.id
        )
    };
    set_status(&db, "copa", "check_in").await;
    assert_eq!(
        rejected(exec(&db, &insert("null")).await),
        rule(CHECK, "participants_check_in_on_entry")
    );
    assert_eq!(exec(&db, &insert("now()")).await.unwrap(), 1);
    // Desfazer o check-in depois continua valendo.
    assert_eq!(
        exec(&db, "update participants set checked_in_at = null")
            .await
            .unwrap(),
        1
    );
    exec(&db, "delete from participants").await.unwrap();
    set_status(&db, "copa", "registration").await;
    assert_eq!(exec(&db, &insert("null")).await.unwrap(), 1);
}
