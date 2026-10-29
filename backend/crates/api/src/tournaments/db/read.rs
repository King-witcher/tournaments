//! Leitura de torneios: a lista do tenant e um torneio pelo slug. As duas
//! aplicam a abertura automática do check-in (`Schedule::check_in_due`) antes
//! de devolver, então o status lido já é o que vale agora. O agendador do
//! tempo real abre pelo mesmo caminho (`current_status`) os que ninguém leu.

use chrono::{DateTime, Utc};
use engine::{champions as ec, formats as ef};
use sqlx::{Acquire, PgConnection, PgPool, types::Json};

use super::{StoredTournament, TournamentId, status::open_due_check_in};
use crate::{
    shared::{ApiError, Locale},
    tenants::TenantId,
    tournaments::{
        changes,
        schedule::Schedule,
        setup::{Setup, TournamentMode},
        status::TournamentStatus,
        texts::Texts,
        types::{NameTranslation, TournamentSummary, TournamentTranslation},
        visibility::Visibility,
    },
};

/// O status que vale em `now`: abre o check-in se a janela abriu e grava,
/// junto com o aviso aos inscritos (numa transação própria, ou num savepoint
/// da de quem chama).
pub async fn current_status(
    conn: &mut PgConnection,
    id: TournamentId,
    stored: TournamentStatus,
    schedule: &Schedule,
    reopened_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> Result<TournamentStatus, sqlx::Error> {
    if !schedule.check_in_due(stored, reopened_at, now) {
        return Ok(stored);
    }
    let mut tx = conn.begin().await?;
    if !open_due_check_in(&mut *tx, id, schedule, reopened_at).await? {
        return Ok(stored);
    }
    changes::check_in_opened(&mut tx, id).await?;
    tx.commit().await?;
    Ok(TournamentStatus::CheckIn)
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
    let mut conn = db.acquire().await?;
    let rows = sqlx::query!(
        r#"select t.id, t.slug, t.locale, t.name, t.mode, t.status, t.visibility,
                  t.max_participants, t.starts_at, t.check_in_minutes,
                  t.registration_reopened_at, t.created_at,
                  (select count(*) from participants p where p.tournament_id = t.id) as "participants!",
                  exists(select 1 from participants p
                         where p.tournament_id = t.id and p.user_id = $2) as "registered!",
                  exists(select 1 from participants p
                         where p.tournament_id = t.id and p.user_id = $2
                           and p.checked_in_at is not null) as "checked_in!",
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
    .fetch_all(&mut *conn)
    .await?;

    let mut list = Vec::with_capacity(rows.len());
    for r in rows {
        let schedule = Schedule {
            starts_at: r.starts_at,
            check_in_minutes: r.check_in_minutes,
        };
        let stored = TournamentStatus::from_db(&r.status)?;
        let status = current_status(
            &mut conn,
            TournamentId(r.id),
            stored,
            &schedule,
            r.registration_reopened_at,
            Utc::now(),
        )
        .await?;
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
        list.push(TournamentSummary {
            slug: r.slug,
            locale: Locale::from_db(&r.locale)?,
            name: r.name,
            mode: TournamentMode::from_db(&r.mode)?,
            status,
            visibility: Visibility::from_db(&r.visibility)?,
            max_participants: r.max_participants,
            participants: r.participants,
            registered: r.registered,
            checked_in: r.checked_in,
            starts_at: r.starts_at,
            translations,
            created_at: r.created_at,
        });
    }
    Ok(list)
}

/// O torneio `slug` do tenant, com fases, traduções e o número de inscritos.
/// `viewer` é quem vê (`None` sem login), para `registered` e `checked_in`.
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
                  t.champion_rules as "champion_rules: Json<ec::ChampionRules>",
                  t.starts_at, t.check_in_minutes, t.registration_reopened_at, t.contest_minutes,
                  t.counts_for_rating, t.created_at,
                  (select count(*) from participants p where p.tournament_id = t.id) as "participants!",
                  exists(select 1 from participants p
                         where p.tournament_id = t.id and p.user_id = $3) as "registered!",
                  exists(select 1 from participants p
                         where p.tournament_id = t.id and p.user_id = $3
                           and p.checked_in_at is not null) as "checked_in!"
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
    let id = TournamentId(row.id);
    let schedule = Schedule {
        starts_at: row.starts_at,
        check_in_minutes: row.check_in_minutes,
    };
    let stored = TournamentStatus::from_db(&row.status)?;
    let status = current_status(
        &mut *conn,
        id,
        stored,
        &schedule,
        row.registration_reopened_at,
        Utc::now(),
    )
    .await?;

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
        id,
        tenant,
        slug: row.slug,
        locale: Locale::from_db(&row.locale)?,
        texts: Texts {
            name: row.name,
            description: row.description,
            rules: row.rules,
        },
        mode: TournamentMode::from_db(&row.mode)?,
        status,
        visibility: Visibility::from_db(&row.visibility)?,
        setup: Setup {
            max_participants: row.max_participants,
            champion_rules: row.champion_rules.0,
            stages: stages.into_iter().map(|s| s.0).collect(),
        },
        schedule,
        contest_minutes: row.contest_minutes,
        counts_for_rating: row.counts_for_rating,
        translations,
        participants: row.participants,
        registered: row.registered,
        checked_in: row.checked_in,
        created_at: row.created_at,
    }))
}

/// Slugs dos torneios do tenant que a saída de `user_id` pode mudar: os que
/// não encerraram (ele pode estar se inscrevendo agora) e os encerrados em que
/// ele está inscrito (uma correção pode reabrir), na ordem dos ids, para quem
/// trava vários travar sempre na mesma ordem.
pub async fn slugs_for_member(
    conn: &mut PgConnection,
    tenant: TenantId,
    user_id: i64,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select t.slug from tournaments t
           where t.tenant_id = $1
             and (t.status <> 'finished'
                  or exists(select 1 from participants p
                            where p.tournament_id = t.id and p.user_id = $2))
           order by t.id"#,
        tenant.get(),
        user_id
    )
    .fetch_all(conn)
    .await
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
