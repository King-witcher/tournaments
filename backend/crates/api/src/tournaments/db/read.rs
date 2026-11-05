//! Leitura de torneios: a lista do tenant (ou de vários, `list_in`) e um
//! torneio pelo slug. As duas
//! aplicam a abertura automática do check-in (`Schedule::check_in_due`) antes
//! de devolver, então o status lido já é o que vale agora. O agendador do
//! tempo real abre pelo mesmo caminho (`current_status`) os que ninguém leu.

use chrono::{DateTime, Utc};
use engine::core::MapId;
use sqlx::{Acquire, PgConnection, PgPool, types::Json};

use super::{
    StoredTournament, TournamentId,
    columns::{ChampionColumns, StageRow},
    status::open_due_check_in,
};
use crate::{
    shared::{ApiError, Locale, public_url},
    tenants::TenantId,
    tournaments::{
        admission::Admission,
        changes,
        schedule::Schedule,
        setup::{Setup, TournamentMode},
        status::TournamentStatus,
        team_rules::{TeamActor, TeamPicker, TeamRules},
        texts::Texts,
        types::{NameTranslation, TournamentSummary, TournamentTranslation},
        veto::{MapMode, PickOrder, PoolMap, VetoOrder, VetoSetup},
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
    now: DateTime<Utc>,
) -> Result<TournamentStatus, sqlx::Error> {
    if !schedule.check_in_due(stored, now) {
        return Ok(stored);
    }
    let mut tx = conn.begin().await?;
    if !open_due_check_in(&mut *tx, id, schedule).await? {
        return Ok(stored);
    }
    changes::check_in_opened(&mut tx, id).await?;
    tx.commit().await?;
    Ok(TournamentStatus::CheckIn)
}

/// Torneios do tenant que `viewer` vê, do mais novo ao mais antigo. Com
/// `see_all` (organizer e owner), todos; sem, nem rascunhos (`visibility::can_see`).
pub async fn list(
    db: &PgPool,
    tenant: TenantId,
    viewer: i64,
    see_all: bool,
) -> Result<Vec<TournamentSummary>, ApiError> {
    let drafts: &[TenantId] = if see_all { &[tenant] } else { &[] };
    let mut conn = db.acquire().await?;
    let listed = list_in(&mut conn, &[tenant], drafts, viewer).await?;
    Ok(listed.into_iter().map(|l| l.summary).collect())
}

/// Um torneio da lista, com o id interno e o tenant: para quem junta as
/// comunidades (`/me/tournaments`) e depois lê mais de alguns deles.
pub struct Listed {
    pub id: TournamentId,
    pub tenant: TenantId,
    pub summary: TournamentSummary,
}

/// Como `list`, dos tenants `tenants` de uma vez (uma query para todas as
/// comunidades de quem vê): os rascunhos só dos de `drafts` (onde `viewer` é
/// organizer ou owner). Do mais novo ao mais antigo, misturando os tenants.
pub async fn list_in(
    conn: &mut PgConnection,
    tenants: &[TenantId],
    drafts: &[TenantId],
    viewer: i64,
) -> Result<Vec<Listed>, ApiError> {
    let ids = |ts: &[TenantId]| ts.iter().map(|t| t.get()).collect::<Vec<_>>();
    let rows = sqlx::query!(
        r#"select t.id, t.tenant_id, t.slug, t.locale, t.name, t.mode, t.team_size, t.status,
                  t.admission, t.max_participants, t.starts_at, t.check_in_minutes, t.created_at,
                  t.cover_key,
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
           where t.tenant_id = any($1)
             and (t.status <> 'draft' or t.tenant_id = any($3))
           group by t.id
           order by t.created_at desc, t.id desc"#,
        &ids(tenants),
        viewer,
        &ids(drafts)
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
        let id = TournamentId(r.id);
        let status = current_status(&mut *conn, id, stored, &schedule, Utc::now()).await?;
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
        // O `TenantId` é o de quem chamou (conferido), não um lido do banco.
        let tenant = *tenants
            .iter()
            .find(|t| t.get() == r.tenant_id)
            .ok_or_else(ApiError::internal)?;
        list.push(Listed {
            id,
            tenant,
            summary: TournamentSummary {
                slug: r.slug,
                locale: Locale::from_db(&r.locale)?,
                name: r.name,
                mode: TournamentMode::from_db(&r.mode)?,
                team_size: i32::from(r.team_size),
                status,
                admission: Admission::from_db(&r.admission)?,
                max_participants: r.max_participants,
                participants: r.participants,
                registered: r.registered,
                checked_in: r.checked_in,
                starts_at: r.starts_at,
                translations,
                created_at: r.created_at,
                cover_url: r.cover_key.as_deref().and_then(public_url),
            },
        });
    }
    Ok(list)
}

/// Um mapa do pool, como vem do `jsonb_agg` de `find`.
#[derive(serde::Deserialize, Debug)]
struct PoolMapRow {
    id: i64,
    slug: String,
    name: String,
}

/// Uma tradução, como vem do `jsonb_agg` de `find`.
#[derive(serde::Deserialize, Debug)]
struct TranslationRow {
    locale: String,
    name: String,
    description: String,
    rules: String,
}

/// O torneio `slug` do tenant, com fases, traduções e o número de inscritos.
/// `viewer` é quem vê (`None` sem login), para `registered` e `checked_in`.
/// Não confere se ele pode ver: ver `visibility::find_visible`. Uma query
/// só (as fases, o pool e as traduções vêm agregadas em JSON): ela está em
/// toda rota de torneio.
pub async fn find(
    conn: &mut PgConnection,
    tenant: TenantId,
    slug: &str,
    viewer: Option<i64>,
) -> Result<Option<StoredTournament>, ApiError> {
    let Some(row) = sqlx::query!(
        r#"select t.id, t.slug, t.locale, t.name, t.description, t.rules, t.mode, t.status,
                  t.admission, t.max_participants, t.bans_per_series, t.mirror, t.same_team,
                  t.no_repeat_who, t.no_repeat_scope,
                  t.starts_at, t.check_in_minutes, t.contest_minutes,
                  t.counts_for_rating, t.map_mode, t.champion_phase, t.veto_seconds,
                  t.screenshots_required, t.created_at, t.team_size, t.team_pick_seconds, t.veto_order,
                  t.pick_order, t.started_at, t.cover_key, t.veto_by, t.ban_by, t.pick_by,
                  t.report_by,
                  (select n.slug from tenants n where n.id = t.tenant_id) as "tenant_slug!",
                  (select count(*) from participants p where p.tournament_id = t.id) as "participants!",
                  exists(select 1 from participants p
                         where p.tournament_id = t.id and p.user_id = $3) as "registered!",
                  exists(select 1 from participants p
                         where p.tournament_id = t.id and p.user_id = $3
                           and p.checked_in_at is not null) as "checked_in!",
                  coalesce((select jsonb_agg(s order by s.position)
                            from (select position, format, best_of, finals_best_of, bracket,
                                         grand_final_reset, rounds, tiebreakers
                                  from tournament_stages
                                  where tournament_id = t.id) s), '[]')
                      as "stages!: Json<Vec<StageRow>>",
                  coalesce((select jsonb_agg(jsonb_build_object('id', m.id, 'slug', m.slug,
                                                                'name', m.name)
                                             order by p.position)
                            from tournament_maps p
                            join maps m on m.id = p.map_id
                            where p.tournament_id = t.id), '[]')
                      as "map_pool!: Json<Vec<PoolMapRow>>",
                  coalesce((select jsonb_agg(jsonb_build_object('locale', r.locale,
                                                                'name', r.name,
                                                                'description', r.description,
                                                                'rules', r.rules)
                                             order by r.locale)
                            from tournament_translations r
                            where r.tournament_id = t.id), '[]')
                      as "translations!: Json<Vec<TranslationRow>>"
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
    let status = current_status(&mut *conn, id, stored, &schedule, Utc::now()).await?;

    let map_pool: Vec<PoolMap> = row
        .map_pool
        .0
        .into_iter()
        .map(|m| PoolMap {
            id: MapId(m.id),
            slug: m.slug,
            name: m.name,
        })
        .collect();
    let translations = row
        .translations
        .0
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
    let stages = row
        .stages
        .0
        .iter()
        .map(StageRow::read)
        .collect::<Result<_, _>>()?;
    let champion_rules = ChampionColumns {
        bans_per_series: row.bans_per_series,
        mirror: row.mirror,
        same_team: row.same_team,
        no_repeat_who: row.no_repeat_who,
        no_repeat_scope: row.no_repeat_scope,
    }
    .read()?;

    Ok(Some(StoredTournament {
        id,
        tenant,
        tenant_slug: row.tenant_slug,
        slug: row.slug,
        locale: Locale::from_db(&row.locale)?,
        texts: Texts {
            name: row.name,
            description: row.description,
            rules: row.rules,
        },
        status,
        admission: Admission::from_db(&row.admission)?,
        setup: Setup {
            mode: TournamentMode::from_db(&row.mode)?,
            team_size: i32::from(row.team_size),
            max_participants: row.max_participants,
            champion_rules,
            stages,
            veto: VetoSetup {
                map_mode: MapMode::from_db(&row.map_mode)?,
                order: VetoOrder::from_db(&row.veto_order)?,
                pool: map_pool.iter().map(|m| m.id).collect(),
                champion_phase: row.champion_phase,
                pick_order: PickOrder::from_db(&row.pick_order)?,
                step_seconds: row.veto_seconds,
                team_pick_seconds: row.team_pick_seconds,
            },
            team_rules: TeamRules {
                veto: TeamActor::from_db(&row.veto_by)?,
                ban: TeamActor::from_db(&row.ban_by)?,
                pick: TeamPicker::from_db(&row.pick_by)?,
                report: TeamActor::from_db(&row.report_by)?,
            },
        },
        schedule,
        contest_minutes: row.contest_minutes,
        counts_for_rating: row.counts_for_rating,
        map_pool,
        screenshots_required: row.screenshots_required,
        translations,
        participants: row.participants,
        registered: row.registered,
        checked_in: row.checked_in,
        created_at: row.created_at,
        started_at: row.started_at,
        cover_key: row.cover_key,
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
