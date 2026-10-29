//! SQL do rating: a configuração, o ranking, os mapas e os ajustes de um
//! membro aqui; o recálculo (marcas, lock, histórico e cache) em `cache`.

mod cache;

use engine::rating::{EloConfig, PROVISIONAL_GAMES};
use sqlx::{PgConnection, PgExecutor, PgPool};

use super::{
    history::Modality,
    types::{
        AdjustmentAuthor, RankingEntry, RatedGame, RatedOpponent, RatedTournament, RatingAdjustment,
    },
};
use crate::{
    shared::{ApiError, Locale},
    tenants::TenantId,
    tournaments::NameTranslation,
};

pub use cache::{consume_dirty, dirty_tenants, is_dirty, lock, read, sources, write};

/// O Elo do tenant; `None` sem configuração gravada (vale o padrão).
pub async fn settings<'e>(
    db: impl PgExecutor<'e>,
    tenant: TenantId,
) -> Result<Option<EloConfig>, sqlx::Error> {
    sqlx::query_as!(
        EloConfig,
        "select initial, k_min, k_max, k_decay from rating_settings where tenant_id = $1",
        tenant.get()
    )
    .fetch_optional(db)
    .await
}

/// Grava o Elo do tenant, com quem mudou.
pub async fn put_settings(
    conn: &mut PgConnection,
    tenant: TenantId,
    config: &EloConfig,
    by: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"insert into rating_settings (tenant_id, initial, k_min, k_max, k_decay, updated_by)
           values ($1, $2, $3, $4, $5, $6)
           on conflict (tenant_id) do update
           set initial = excluded.initial, k_min = excluded.k_min, k_max = excluded.k_max,
               k_decay = excluded.k_decay, updated_by = excluded.updated_by, updated_at = now()"#,
        tenant.get(),
        config.initial,
        config.k_min,
        config.k_max,
        config.k_decay,
        by
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// O ranking: os membros ativos com rating, do maior para o menor.
pub async fn ranking(
    db: &PgPool,
    tenant: TenantId,
    modality: Modality,
) -> Result<Vec<RankingEntry>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"select rank() over (order by r.rating desc) as "position!", r.user_id, m.nickname,
                  u.display_name, u.avatar_url, r.rating, r.games, r.wins, r.losses,
                  r.last_game_at
           from ratings r
           join tenant_members m on m.tenant_id = r.tenant_id and m.user_id = r.user_id
           join users u on u.id = r.user_id
           where r.tenant_id = $1 and r.modality = $2 and m.left_at is null
           order by r.rating desc, r.games desc, m.nickname, r.user_id"#,
        tenant.get(),
        modality.as_str()
    )
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| RankingEntry {
            position: r.position,
            user_id: r.user_id,
            nickname: r.nickname,
            display_name: r.display_name,
            avatar_url: r.avatar_url,
            rating: r.rating,
            provisional: r.games < PROVISIONAL_GAMES as i32,
            games: r.games,
            wins: r.wins,
            losses: r.losses,
            last_game_at: r.last_game_at,
        })
        .collect())
}

/// Até `limit` mapas que contaram para `user`, do mais recente, só dos
/// torneios que `viewer` vê (`see_all`: organizer e owner veem todos; os
/// outros, os públicos e os privados em que estão inscritos, como na lista da
/// comunidade).
pub async fn recent_games(
    db: &PgPool,
    tenant: TenantId,
    modality: Modality,
    user: i64,
    viewer: i64,
    see_all: bool,
    limit: i64,
) -> Result<Vec<RatedGame>, ApiError> {
    let rows = sqlx::query!(
        r#"select g.match_key, g.number, mp.name as "map?", h.won as "won!", h.delta,
                  h.rating_after, h.at, t.slug, t.locale, t.name,
                  coalesce((select array_agg(tr.locale order by tr.locale)
                            from tournament_translations tr where tr.tournament_id = t.id),
                           '{}') as "translation_locales!",
                  coalesce((select array_agg(tr.name order by tr.locale)
                            from tournament_translations tr where tr.tournament_id = t.id),
                           '{}') as "translation_names!",
                  o.user_id as opponent_id, o.nickname as opponent_nickname,
                  ou.avatar_url as opponent_avatar, o.left_at is null as "opponent_active!"
           from rating_history h
           join match_games g on g.id = h.game_id and g.tenant_id = h.tenant_id
           join tournaments t on t.id = g.tournament_id
           left join maps mp on mp.id = g.map_id
           join tenant_members o on o.tenant_id = h.tenant_id and o.user_id = h.opponent_id
           join users ou on ou.id = o.user_id
           where h.tenant_id = $1 and h.modality = $2 and h.user_id = $3
             and ($5 or (t.status <> 'draft'
                         and (t.visibility = 'public'
                              or exists(select 1 from participants p
                                        where p.tournament_id = t.id and p.user_id = $4))))
           order by h.seq desc
           limit $6"#,
        tenant.get(),
        modality.as_str(),
        user,
        viewer,
        see_all,
        limit
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
            Ok(RatedGame {
                tournament: RatedTournament {
                    slug: r.slug,
                    locale: Locale::from_db(&r.locale)?,
                    name: r.name,
                    translations,
                },
                match_key: r.match_key,
                game: r.number,
                map: r.map,
                opponent: RatedOpponent {
                    user_id: r.opponent_id,
                    nickname: r.opponent_nickname,
                    avatar_url: r.opponent_avatar,
                    active: r.opponent_active,
                },
                won: r.won,
                delta: r.delta,
                rating_after: r.rating_after,
                at: r.at,
            })
        })
        .collect()
}

/// Os ajustes de `user`, do mais recente ao mais antigo.
pub async fn adjustments(
    db: &PgPool,
    tenant: TenantId,
    modality: Modality,
    user: i64,
) -> Result<Vec<RatingAdjustment>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"select a.id, a.delta, a.reason, a.created_at, a.created_by,
                  c.nickname as "created_by_nickname?"
           from rating_adjustments a
           left join tenant_members c on c.tenant_id = a.tenant_id and c.user_id = a.created_by
           where a.tenant_id = $1 and a.modality = $2 and a.user_id = $3
           order by a.created_at desc, a.id desc"#,
        tenant.get(),
        modality.as_str(),
        user
    )
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| RatingAdjustment {
            id: r.id,
            delta: r.delta,
            reason: r.reason,
            created_by: r
                .created_by
                .zip(r.created_by_nickname)
                .map(|(user_id, nickname)| AdjustmentAuthor { user_id, nickname }),
            created_at: r.created_at,
        })
        .collect())
}

/// Grava o ajuste de `user` feito por `by`.
pub async fn insert_adjustment(
    conn: &mut PgConnection,
    tenant: TenantId,
    modality: Modality,
    user: i64,
    delta: i32,
    reason: &str,
    by: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"insert into rating_adjustments (tenant_id, modality, user_id, delta, reason, created_by)
           values ($1, $2, $3, $4, $5, $6)
           returning id"#,
        tenant.get(),
        modality.as_str(),
        user,
        delta,
        reason,
        by
    )
    .fetch_one(conn)
    .await
}
