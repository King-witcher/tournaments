//! SQL do que espera a organização: uma query por lista, de todas as
//! comunidades em que `user` é organizer ou owner ativo, com o nome das
//! traduções de cada torneio na mesma linha.

use chrono::{DateTime, Utc};
use engine::formats::{self as ef, elimination::EliminationKind};
use sqlx::{PgPool, types::Json};

use super::types::{ContestedGame, PendingTournament, StartDue};
use crate::{
    shared::{ApiError, Locale},
    tournaments::{NameTranslation, TournamentStatus},
};

/// Os mapas contestados dos torneios das comunidades que `user` organiza, do
/// contestado há mais tempo ao mais recente. Mapa contestado só existe em
/// partida pronta de torneio em andamento: o `settle` anula os que deixam de
/// valer, e desfazer o início apaga todos.
pub async fn contested(db: &PgPool, user: i64) -> Result<Vec<ContestedGame>, ApiError> {
    sqlx::query!(
        r#"select te.slug as tenant_slug, te.name as tenant_name, t.slug, t.name,
                  coalesce((select array_agg(tr.locale order by tr.locale)
                            from tournament_translations tr where tr.tournament_id = t.id),
                           '{}') as "translation_locales!",
                  coalesce((select array_agg(tr.name order by tr.locale)
                            from tournament_translations tr where tr.tournament_id = t.id),
                           '{}') as "translation_names!",
                  s.config as "config: Json<ef::StageFormat>",
                  g.match_key, g.number, mp.name as "map?", c.nickname as "contested_by?",
                  g.contested_at as "contested_at!"
           from tenant_members m
           join tenants te on te.id = m.tenant_id
           join tournaments t on t.tenant_id = m.tenant_id
           join match_games g on g.tournament_id = t.id
           join tournament_stages s on s.tournament_id = g.tournament_id and s.position = g.stage
           left join maps mp on mp.id = g.map_id
           left join tenant_members c on c.tenant_id = g.tenant_id and c.user_id = g.contested_by
           where m.user_id = $1 and m.left_at is null and m.role in ('organizer', 'owner')
             and g.status = 'contested'
           order by g.contested_at, g.id"#,
        user
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|r| {
        let ef::StageFormat::Elimination(config) = r.config.0;
        Ok(ContestedGame {
            tournament: PendingTournament {
                tenant_slug: r.tenant_slug,
                tenant_name: r.tenant_name,
                slug: r.slug,
                name: r.name,
                translations: translations(r.translation_locales, r.translation_names)?,
            },
            key: r.match_key,
            single: config.kind == EliminationKind::Single,
            game: r.number,
            map: r.map,
            contested_by: r.contested_by,
            contested_at: r.contested_at,
        })
    })
    .collect()
}

/// Os torneios das comunidades que `user` organiza que estão na inscrição ou
/// no check-in com o início marcado já passado em `now` (o critério do aviso
/// `tournament.start_due`, sem o "uma vez"), do início mais antigo.
pub async fn start_due(
    db: &PgPool,
    user: i64,
    now: DateTime<Utc>,
) -> Result<Vec<StartDue>, ApiError> {
    sqlx::query!(
        r#"select te.slug as tenant_slug, te.name as tenant_name, t.slug, t.name,
                  coalesce((select array_agg(tr.locale order by tr.locale)
                            from tournament_translations tr where tr.tournament_id = t.id),
                           '{}') as "translation_locales!",
                  coalesce((select array_agg(tr.name order by tr.locale)
                            from tournament_translations tr where tr.tournament_id = t.id),
                           '{}') as "translation_names!",
                  t.status, t.starts_at as "starts_at!", t.check_in_minutes,
                  (select count(*) from participants p
                   where p.tournament_id = t.id) as "registered!",
                  (select count(*) from participants p
                   where p.tournament_id = t.id and p.checked_in_at is not null) as "checked_in!"
           from tenant_members m
           join tenants te on te.id = m.tenant_id
           join tournaments t on t.tenant_id = m.tenant_id
           where m.user_id = $1 and m.left_at is null and m.role in ('organizer', 'owner')
             and t.status in ('registration', 'check_in') and t.starts_at <= $2
           order by t.starts_at, t.id"#,
        user,
        now
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|r| {
        Ok(StartDue {
            tournament: PendingTournament {
                tenant_slug: r.tenant_slug,
                tenant_name: r.tenant_name,
                slug: r.slug,
                name: r.name,
                translations: translations(r.translation_locales, r.translation_names)?,
            },
            status: TournamentStatus::from_db(&r.status)?,
            starts_at: r.starts_at,
            has_check_in: r.check_in_minutes.is_some(),
            registered: r.registered,
            checked_in: r.checked_in,
        })
    })
    .collect()
}

fn translations(
    locales: Vec<String>,
    names: Vec<String>,
) -> Result<Vec<NameTranslation>, ApiError> {
    locales
        .iter()
        .zip(names)
        .map(|(locale, name)| {
            Ok(NameTranslation {
                locale: Locale::from_db(locale)?,
                name,
            })
        })
        .collect()
}
