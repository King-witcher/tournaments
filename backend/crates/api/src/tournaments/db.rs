use chrono::{DateTime, Utc};
use engine::{champions as ec, formats as ef};
use sqlx::{PgConnection, PgExecutor, PgPool, types::Json};

use super::{
    setup::{Setup, TournamentMode},
    status::TournamentStatus,
    texts::Texts,
    types::{NameTranslation, TournamentSummary, TournamentTranslation},
    visibility::Visibility,
};
use crate::{
    shared::{ApiError, Locale},
    tenants::TenantId,
};

/// Id interno de um torneio. Só sai do banco por `find`/`find_for_update`,
/// que filtram pelo tenant; quem recebe um `TournamentId` já passou por lá.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TournamentId(i64);

impl TournamentId {
    pub const fn get(self) -> i64 {
        self.0
    }
}

pub struct StoredTournament {
    pub id: TournamentId,
    pub slug: String,
    pub locale: Locale,
    pub texts: Texts,
    pub mode: TournamentMode,
    pub status: TournamentStatus,
    pub visibility: Visibility,
    pub setup: Setup,
    pub translations: Vec<TournamentTranslation>,
    /// Inscritos agora.
    pub participants: i64,
    /// Se o `viewer` passado a `find` está inscrito.
    pub registered: bool,
    pub created_at: DateTime<Utc>,
}

pub struct NewTournament<'a> {
    pub slug: &'a str,
    pub locale: Locale,
    pub texts: &'a Texts,
    pub mode: TournamentMode,
    pub visibility: Visibility,
    pub setup: &'a Setup,
}

/// Campeões ativos: o pool de todo torneio, por enquanto.
pub async fn champion_pool<'e>(db: impl PgExecutor<'e>) -> Result<u32, ApiError> {
    let count = sqlx::query_scalar!(r#"select count(*) as "count!" from champions where active"#)
        .fetch_one(db)
        .await?;
    u32::try_from(count).map_err(|_| ApiError::internal())
}

/// Cria o torneio como rascunho, com as fases.
pub async fn create(db: &PgPool, tenant: TenantId, new: NewTournament<'_>) -> Result<(), ApiError> {
    let mut tx = db.begin().await?;
    let id = sqlx::query_scalar!(
        r#"insert into tournaments
               (tenant_id, slug, locale, name, description, rules, mode, visibility,
                max_participants, champion_rules)
           values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
           on conflict (tenant_id, slug) do nothing
           returning id"#,
        tenant.get(),
        new.slug,
        new.locale.as_str(),
        new.texts.name,
        new.texts.description,
        new.texts.rules,
        new.mode.as_str(),
        new.visibility.as_str(),
        new.setup.max_participants,
        Json(new.setup.champion_rules) as _
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(ApiError::conflict("tournament.slug_taken"))?;
    insert_stages(&mut tx, TournamentId(id), &new.setup.stages).await?;
    tx.commit().await?;
    Ok(())
}

async fn insert_stages(
    conn: &mut PgConnection,
    id: TournamentId,
    stages: &[ef::StageFormat],
) -> Result<(), sqlx::Error> {
    for (position, stage) in (1..).zip(stages) {
        sqlx::query!(
            "insert into tournament_stages (tournament_id, position, config) values ($1, $2, $3)",
            id.0,
            position as i16,
            Json(stage) as _
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

/// Torneios do tenant que `viewer` vê, do mais novo ao mais antigo. Com
/// `see_all` (organizer e owner), todos; sem, nem rascunhos nem privados em
/// que ele não está inscrito (`visibility::can_see`).
pub async fn list(
    db: &PgPool,
    tenant: TenantId,
    viewer: i64,
    see_all: bool,
) -> Result<Vec<TournamentSummary>, ApiError> {
    let rows = sqlx::query!(
        r#"select t.slug, t.locale, t.name, t.mode, t.status, t.visibility, t.max_participants,
                  t.created_at,
                  (select count(*) from participants p where p.tournament_id = t.id) as "participants!",
                  exists(select 1 from participants p
                         where p.tournament_id = t.id and p.user_id = $2) as "registered!",
                  coalesce(array_agg(tr.locale order by tr.locale) filter (where tr.locale is not null),
                           '{}') as "translation_locales!",
                  coalesce(array_agg(tr.name order by tr.locale) filter (where tr.locale is not null),
                           '{}') as "translation_names!"
           from tournaments t
           left join tournament_translations tr on tr.tournament_id = t.id
           where t.tenant_id = $1
             and ($3 or (t.status <> 'draft'
                         and (t.visibility = 'public'
                              or exists(select 1 from participants p
                                        where p.tournament_id = t.id and p.user_id = $2))))
           group by t.id
           order by t.created_at desc, t.id desc"#,
        tenant.get(),
        viewer,
        see_all
    )
    .fetch_all(db)
    .await?;

    rows.into_iter()
        .map(|r| {
            let translations = r
                .translation_locales
                .iter()
                .zip(r.translation_names)
                .map(|(locale, name)| {
                    Ok(NameTranslation {
                        locale: Locale::from_db(locale)?,
                        name,
                    })
                })
                .collect::<Result<_, ApiError>>()?;
            Ok(TournamentSummary {
                slug: r.slug,
                locale: Locale::from_db(&r.locale)?,
                name: r.name,
                mode: TournamentMode::from_db(&r.mode)?,
                status: TournamentStatus::from_db(&r.status)?,
                visibility: Visibility::from_db(&r.visibility)?,
                max_participants: r.max_participants,
                participants: r.participants,
                registered: r.registered,
                translations,
                created_at: r.created_at,
            })
        })
        .collect()
}

/// O torneio `slug` do tenant, com fases, traduções e o número de inscritos.
/// `viewer` é quem vê (`None` sem login), para `StoredTournament::registered`.
/// Não confere a visibilidade: ver `visibility::find_visible`.
pub async fn find(
    conn: &mut PgConnection,
    tenant: TenantId,
    slug: &str,
    viewer: Option<i64>,
) -> Result<Option<StoredTournament>, ApiError> {
    let Some(row) = sqlx::query!(
        r#"select t.id, t.slug, t.locale, t.name, t.description, t.rules, t.mode, t.status,
                  t.visibility, t.max_participants,
                  t.champion_rules as "champion_rules: Json<ec::ChampionRules>", t.created_at,
                  (select count(*) from participants p where p.tournament_id = t.id) as "participants!",
                  exists(select 1 from participants p
                         where p.tournament_id = t.id and p.user_id = $3) as "registered!"
           from tournaments t
           where t.tenant_id = $1 and t.slug = $2"#,
        tenant.get(),
        slug,
        viewer
    )
    .fetch_optional(&mut *conn)
    .await?
    else {
        return Ok(None);
    };

    let stages = sqlx::query_scalar!(
        r#"select config as "config: Json<ef::StageFormat>"
           from tournament_stages
           where tournament_id = $1
           order by position"#,
        row.id
    )
    .fetch_all(&mut *conn)
    .await?;
    let translations = sqlx::query!(
        r#"select locale, name, description, rules
           from tournament_translations
           where tournament_id = $1
           order by locale"#,
        row.id
    )
    .fetch_all(&mut *conn)
    .await?
    .into_iter()
    .map(|t| {
        Ok(TournamentTranslation {
            locale: Locale::from_db(&t.locale)?,
            name: t.name,
            description: t.description,
            rules: t.rules,
        })
    })
    .collect::<Result<_, ApiError>>()?;

    Ok(Some(StoredTournament {
        id: TournamentId(row.id),
        slug: row.slug,
        locale: Locale::from_db(&row.locale)?,
        texts: Texts {
            name: row.name,
            description: row.description,
            rules: row.rules,
        },
        mode: TournamentMode::from_db(&row.mode)?,
        status: TournamentStatus::from_db(&row.status)?,
        visibility: Visibility::from_db(&row.visibility)?,
        setup: Setup {
            max_participants: row.max_participants,
            champion_rules: row.champion_rules.0,
            stages: stages.into_iter().map(|s| s.0).collect(),
        },
        translations,
        participants: row.participants,
        registered: row.registered,
        created_at: row.created_at,
    }))
}

/// Como `find`, travando a linha do torneio até o fim da transação: status,
/// configuração e inscritos não mudam entre a checagem e a gravação (toda
/// inscrição trava o torneio antes).
pub async fn find_for_update(
    conn: &mut PgConnection,
    tenant: TenantId,
    slug: &str,
    viewer: Option<i64>,
) -> Result<Option<StoredTournament>, ApiError> {
    let found = sqlx::query_scalar!(
        "select id from tournaments where tenant_id = $1 and slug = $2 for update",
        tenant.get(),
        slug
    )
    .fetch_optional(&mut *conn)
    .await?;
    if found.is_none() {
        return Ok(None);
    }
    find(conn, tenant, slug, viewer).await
}

/// Grava os textos, a visibilidade (se veio) e a nova configuração (se veio:
/// vagas, campeões e fases).
pub async fn update(
    conn: &mut PgConnection,
    id: TournamentId,
    texts: &Texts,
    visibility: Option<Visibility>,
    setup: Option<&Setup>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"update tournaments
           set name = $2, description = $3, rules = $4, visibility = coalesce($5, visibility),
               updated_at = now()
           where id = $1"#,
        id.0,
        texts.name,
        texts.description,
        texts.rules,
        visibility.map(Visibility::as_str)
    )
    .execute(&mut *conn)
    .await?;
    let Some(setup) = setup else {
        return Ok(());
    };
    sqlx::query!(
        "update tournaments set max_participants = $2, champion_rules = $3 where id = $1",
        id.0,
        setup.max_participants,
        Json(setup.champion_rules) as _
    )
    .execute(&mut *conn)
    .await?;
    sqlx::query!(
        "delete from tournament_stages where tournament_id = $1",
        id.0
    )
    .execute(&mut *conn)
    .await?;
    insert_stages(conn, id, &setup.stages).await
}

/// Rascunho → inscrição.
pub async fn publish(conn: &mut PgConnection, id: TournamentId) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update tournaments set status = 'registration', updated_at = now() where id = $1",
        id.0
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Exclui o torneio se ainda for rascunho. `false` se não era.
pub async fn delete_draft(db: &PgPool, id: TournamentId) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "delete from tournaments where id = $1 and status = 'draft'",
        id.0
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Cria ou substitui a tradução de `locale`.
pub async fn put_translation(
    db: &PgPool,
    id: TournamentId,
    locale: Locale,
    texts: &Texts,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"insert into tournament_translations (tournament_id, locale, name, description, rules)
           values ($1, $2, $3, $4, $5)
           on conflict (tournament_id, locale) do update
           set name = excluded.name, description = excluded.description, rules = excluded.rules"#,
        id.0,
        locale.as_str(),
        texts.name,
        texts.description,
        texts.rules
    )
    .execute(db)
    .await?;
    Ok(())
}

/// `false` se não havia tradução nesse idioma.
pub async fn delete_translation(
    db: &PgPool,
    id: TournamentId,
    locale: Locale,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "delete from tournament_translations where tournament_id = $1 and locale = $2",
        id.0,
        locale.as_str()
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() > 0)
}
