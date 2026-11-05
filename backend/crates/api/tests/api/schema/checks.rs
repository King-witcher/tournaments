//! Coerência e forma: estado, regras de campeões e colunas de cada formato de
//! fase, identidade imutável, vagas, autoria dentro do tenant e do torneio,
//! formato de logins e segredos, o token do link de convite de cada escopo,
//! senha só com email.

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

    // As regras de campeões: faixa, valores e o par da não repetição.
    for (set, name) in [
        ("bans_per_series = 4", "tournaments_bans_per_series"),
        ("bans_per_series = -1", "tournaments_bans_per_series"),
        (
            "no_repeat_who = 'squad', no_repeat_scope = 'series'",
            "tournaments_no_repeat_who",
        ),
        (
            "no_repeat_who = 'player', no_repeat_scope = 'match'",
            "tournaments_no_repeat_scope",
        ),
        ("no_repeat_who = 'player'", "tournaments_no_repeat_pair"),
        ("no_repeat_scope = 'series'", "tournaments_no_repeat_pair"),
    ] {
        let sql = format!("update tournaments set {set}");
        assert_eq!(rejected(exec(&db, &sql).await), rule(CHECK, name), "{sql}");
    }
    exec(
        &db,
        "update tournaments set no_repeat_who = 'team', no_repeat_scope = 'stage'",
    )
    .await
    .unwrap();

    // Cada formato com as suas colunas (a fase nasce em eliminação dupla).
    let single = "format = 'elimination', bracket = 'single', grand_final_reset = null, \
                  best_of = 1, finals_best_of = 3, rounds = null, tiebreakers = null";
    let round_robin = "format = 'round_robin', bracket = null, grand_final_reset = null, \
                       finals_best_of = null, tiebreakers = '{}'";
    let swiss = "format = 'swiss', bracket = null, grand_final_reset = null, \
                 finals_best_of = null, tiebreakers = '{}'";
    // Uma fase de tabela com os desempates `list` (o resto válido).
    let table = |format: &str, list: &str| {
        format!(
            "format = '{format}', bracket = null, grand_final_reset = null, \
             finals_best_of = null, rounds = null, best_of = 3, tiebreakers = {list}"
        )
    };
    for (set, name) in [
        ("format = 'ladder'".to_owned(), "tournament_stages_format"),
        ("best_of = 2".to_owned(), "tournament_stages_best_of"),
        ("best_of = 0".to_owned(), "tournament_stages_best_of"),
        ("best_of = 257".to_owned(), "tournament_stages_best_of"),
        (
            "finals_best_of = 4".to_owned(),
            "tournament_stages_finals_best_of",
        ),
        ("bracket = 'triple'".to_owned(), "tournament_stages_bracket"),
        (
            "finals_best_of = null".to_owned(),
            "tournament_stages_shape",
        ),
        ("bracket = null".to_owned(), "tournament_stages_shape"),
        // Reset só na dupla, e sempre nela.
        ("bracket = 'single'".to_owned(), "tournament_stages_shape"),
        (
            "grand_final_reset = null".to_owned(),
            "tournament_stages_shape",
        ),
        // Rodadas só no suíço.
        ("rounds = 3".to_owned(), "tournament_stages_shape"),
        (
            format!("{round_robin}, rounds = 3"),
            "tournament_stages_shape",
        ),
        // A fase de tabela sem as colunas da eliminação.
        (
            "format = 'round_robin'".to_owned(),
            "tournament_stages_shape",
        ),
        ("format = 'swiss'".to_owned(), "tournament_stages_shape"),
        (format!("{swiss}, rounds = 0"), "tournament_stages_rounds"),
        (format!("{swiss}, rounds = 128"), "tournament_stages_rounds"),
        // Série par só no round robin, e só 2, 4 ou 6.
        ("best_of = 4".to_owned(), "tournament_stages_best_of"),
        (
            "finals_best_of = 6".to_owned(),
            "tournament_stages_finals_best_of",
        ),
        (
            format!("{swiss}, rounds = null, best_of = 2"),
            "tournament_stages_best_of",
        ),
        (
            format!("{swiss}, rounds = null, best_of = 4"),
            "tournament_stages_best_of",
        ),
        (
            format!("{swiss}, rounds = null, best_of = 6"),
            "tournament_stages_best_of",
        ),
        (
            format!("{round_robin}, rounds = null, best_of = 8"),
            "tournament_stages_best_of",
        ),
        // Desempates: só nas fases de tabela, e sempre nelas.
        ("tiebreakers = '{}'".to_owned(), "tournament_stages_shape"),
        (table("round_robin", "null"), "tournament_stages_shape"),
        (table("swiss", "null"), "tournament_stages_shape"),
        // Os que existem, cada um uma vez, numa lista simples.
        (table("swiss", "'{seed}'"), "tournament_stages_tiebreakers"),
        (
            table("swiss", "'{map_diff,head_to_head,map_diff}'"),
            "tournament_stages_tiebreakers",
        ),
        (
            table("swiss", "array['map_diff', null]"),
            "tournament_stages_tiebreakers",
        ),
        (
            table("swiss", "'{{map_diff},{frag_diff}}'"),
            "tournament_stages_tiebreakers",
        ),
        (
            table("swiss", "'[0:0]={map_diff}'"),
            "tournament_stages_tiebreakers",
        ),
        // O Buchholz só no suíço.
        (
            table("round_robin", "'{head_to_head,buchholz}'"),
            "tournament_stages_buchholz_swiss",
        ),
    ] {
        let sql = format!("update tournament_stages set {set}");
        assert_eq!(rejected(exec(&db, &sql).await), rule(CHECK, name), "{sql}");
    }
    for set in [
        format!("{round_robin}, rounds = null"),
        format!("{round_robin}, rounds = null, best_of = 2"),
        format!("{round_robin}, rounds = null, best_of = 4"),
        format!("{round_robin}, rounds = null, best_of = 6"),
        format!("{swiss}, rounds = null, best_of = 3"),
        format!("{swiss}, rounds = 127"),
        table("round_robin", "'{frag_diff,map_diff,head_to_head}'"),
        table("swiss", "'{head_to_head,buchholz,frag_diff,map_diff}'"),
        single.to_owned(),
    ] {
        let sql = format!("update tournament_stages set {set}");
        exec(&db, &sql).await.unwrap();
    }

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
        "insert into tournament_stages
             (tournament_id, position, format, best_of, finals_best_of, bracket)
         select id, 1, 'elimination', 3, 3, 'single' from tournaments",
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
        "insert into tournaments (tenant_id, slug, locale, name, mode, max_participants, bans_per_series, mirror, same_team, invite_token)
         values ($1, 'outra', 'pt', 'Outra', 'duel', 8, 1, true, false, $2)",
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
    translate("en").await.unwrap();
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
