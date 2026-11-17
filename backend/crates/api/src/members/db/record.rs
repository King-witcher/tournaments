//! O retrospecto de um membro, lido direto das séries decididas
//! (`match_results`) e dos mapas confirmados (`match_games` com a escalação de
//! `match_game_players`) de todos os torneios do tenant, com rating ligado ou
//! não. Cada consulta parte dos torneios do tenant, para usar os índices de
//! `match_game_players` (torneio, jogador) e de `participants` (tenant,
//! jogador).
//!
//! As séries de alguém: as que ele jogou (algum mapa confirmado nela, com o
//! lado da escalação) e, nas decididas sem mapa confirmado (W.O.), as do time
//! dele no torneio. A chave que sai é a pública (`s<N>-` da fase 2 em diante).

use serde::Deserialize;
use sqlx::{PgConnection, types::Json};

use super::super::{
    career::{GameTotals, OpponentGames, Series},
    types::{
        GameFilters, GameRating, GameRecord, MapLabel, MemberGame, PersonLabel, SeriesOutcome,
        TournamentLabel,
    },
};
use crate::{
    auth::resolve_avatar,
    matches::MatchKey,
    shared::{ApiError, Locale},
    teams,
    tenants::TenantId,
    tournaments::{NameTranslation, TournamentMode},
};

/// As séries decididas de `user`, da mais recente, só da modalidade `mode`
/// quando há. A hora de uma série é a da primeira decisão: a correção regrava
/// `confirmed_at` mas mantém a posição no replay, então vale a menor hora
/// entre ela e as decididas depois dela no torneio (desempate pelo id).
pub async fn series(
    conn: &mut PgConnection,
    tenant: TenantId,
    user: i64,
    mode: Option<TournamentMode>,
) -> Result<Vec<Series>, ApiError> {
    let rows = sqlx::query!(
        r#"with played as (
               select distinct g.tournament_id, g.stage, g.match_key,
                      case p.side when 'a' then g.side_a else g.side_b end as team
               from tournaments t
               join match_game_players p on p.tournament_id = t.id and p.user_id = $2
               join match_games g on g.id = p.game_id
               where t.tenant_id = $1 and g.status = 'confirmed'
                 and ($3::text is null or t.mode = $3)
           ),
           listed as (
               select r.id, pl.team
               from played pl
               join match_results r on r.tournament_id = pl.tournament_id
                                   and r.stage = pl.stage and r.match_key = pl.match_key
               union all
               select r.id, pa.team_id
               from tournaments t
               join participants pa on pa.tournament_id = t.id and pa.user_id = $2
               join match_results r on r.tournament_id = t.id
                                   and pa.team_id in (r.side_a, r.side_b)
               where t.tenant_id = $1 and ($3::text is null or t.mode = $3)
                 and not exists (
                     select 1 from match_games g
                     where g.tournament_id = r.tournament_id and g.stage = r.stage
                       and g.match_key = r.match_key and g.status = 'confirmed')
           ),
           timed as (
               select r.id,
                      min(r.confirmed_at) over (partition by r.tournament_id
                                                order by r.position desc) as at
               from match_results r
               where r.tournament_id in (select x.tournament_id from match_results x
                                         join listed l on l.id = x.id)
           )
           select r.id, t.slug, t.locale, t.name,
                  coalesce((select json_agg(json_build_object('locale', tr.locale, 'name', tr.name)
                                            order by tr.locale)
                            from tournament_translations tr where tr.tournament_id = t.id),
                           '[]') as "translations!: Json<Vec<NameTranslation>>",
                  r.stage, r.match_key,
                  case when r.side_a = l.team then r.side_b else r.side_a end as "opponent!",
                  r.winner = l.team as won,
                  case when r.side_a = l.team then r.a_wins else r.b_wins end as wins,
                  case when r.side_a = l.team then r.b_wins else r.a_wins end as losses,
                  tm.at as "at!",
                  coalesce(
                      (select array_agg(distinct o.user_id)
                       from match_games g
                       join match_game_players o on o.game_id = g.id
                       where g.tournament_id = r.tournament_id and g.stage = r.stage
                         and g.match_key = r.match_key and g.status = 'confirmed'
                         and (case o.side when 'a' then g.side_a else g.side_b end) <> l.team),
                      (select array_agg(pa.user_id order by pa.user_id)
                       from participants pa
                       where pa.tournament_id = r.tournament_id
                         and pa.team_id = case when r.side_a = l.team then r.side_b
                                               else r.side_a end),
                      '{}') as "opponents!"
           from listed l
           join match_results r on r.id = l.id
           join timed tm on tm.id = r.id
           join tournaments t on t.id = r.tournament_id
           order by tm.at desc, r.id desc"#,
        tenant.get(),
        user,
        mode.map(TournamentMode::as_str)
    )
    .fetch_all(conn)
    .await?;
    rows.into_iter()
        .map(|r| {
            Ok(Series {
                tournament: TournamentLabel {
                    slug: r.slug,
                    locale: Locale::from_db(&r.locale)?,
                    name: r.name,
                    translations: r.translations.0,
                },
                match_key: MatchKey {
                    stage: r.stage,
                    local: r.match_key,
                }
                .public(),
                opponent_team: r.opponent,
                outcome: match r.won {
                    Some(true) => SeriesOutcome::Won,
                    Some(false) => SeriesOutcome::Lost,
                    None => SeriesOutcome::Draw,
                },
                wins: r.wins,
                losses: r.losses,
                at: r.at,
                opponents: r.opponents,
            })
        })
        .collect()
}

/// Os mapas confirmados de `user`: vitórias, derrotas e os frags dos que têm
/// placar.
pub async fn game_totals(
    conn: &mut PgConnection,
    tenant: TenantId,
    user: i64,
) -> Result<GameTotals, sqlx::Error> {
    let r = sqlx::query!(
        r#"select count(*) filter (where g.winner = case p.side when 'a' then g.side_a
                                                               else g.side_b end)::int as "wins!",
                  count(*) filter (where g.winner <> case p.side when 'a' then g.side_a
                                                                else g.side_b end)::int as "losses!",
                  coalesce(sum(case p.side when 'a' then g.a_frags else g.b_frags end), 0)::int
                      as "frags_for!",
                  coalesce(sum(case p.side when 'a' then g.b_frags else g.a_frags end), 0)::int
                      as "frags_against!"
           from tournaments t
           join match_game_players p on p.tournament_id = t.id and p.user_id = $2
           join match_games g on g.id = p.game_id
           where t.tenant_id = $1 and g.status = 'confirmed'"#,
        tenant.get(),
        user
    )
    .fetch_one(conn)
    .await?;
    Ok(GameTotals {
        record: GameRecord {
            wins: r.wins,
            losses: r.losses,
        },
        frags_for: r.frags_for,
        frags_against: r.frags_against,
    })
}

/// Os mapas confirmados de `user` contra cada pessoa que esteve do outro lado,
/// só da modalidade `mode` quando há.
pub async fn games_by_opponent(
    conn: &mut PgConnection,
    tenant: TenantId,
    user: i64,
    mode: Option<TournamentMode>,
) -> Result<Vec<OpponentGames>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"select o.user_id,
                  count(*) filter (where g.winner = case p.side when 'a' then g.side_a
                                                               else g.side_b end)::int as "wins!",
                  count(*) filter (where g.winner <> case p.side when 'a' then g.side_a
                                                                else g.side_b end)::int as "losses!",
                  max(g.confirmed_at) as "last_at!"
           from tournaments t
           join match_game_players p on p.tournament_id = t.id and p.user_id = $2
           join match_games g on g.id = p.game_id
           join match_game_players o on o.game_id = g.id and o.side <> p.side
           where t.tenant_id = $1 and g.status = 'confirmed'
             and ($3::text is null or t.mode = $3)
           group by o.user_id"#,
        tenant.get(),
        user,
        mode.map(TournamentMode::as_str)
    )
    .fetch_all(conn)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| OpponentGames {
            user_id: r.user_id,
            record: GameRecord {
                wins: r.wins,
                losses: r.losses,
            },
            last_at: r.last_at,
        })
        .collect())
}

/// As pessoas `users` no tenant, ativas ou não (quem saiu continua com o
/// apelido; a conta excluída, com o vazio).
pub async fn people(
    conn: &mut PgConnection,
    tenant: TenantId,
    users: &[i64],
) -> Result<Vec<PersonLabel>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"select m.user_id, m.nickname, coalesce(u.avatar_key, u.avatar_url) as avatar_url,
                  m.left_at is null as "active!"
           from tenant_members m
           join users u on u.id = m.user_id
           where m.tenant_id = $1 and m.user_id = any($2)"#,
        tenant.get(),
        users
    )
    .fetch_all(conn)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| PersonLabel {
            user_id: r.user_id,
            nickname: r.nickname,
            avatar_url: resolve_avatar(r.avatar_url),
            active: r.active,
        })
        .collect())
}

/// Os filtros de uma página de mapas.
pub struct GamesFilter<'a> {
    pub tournament: Option<&'a str>,
    pub map: Option<&'a str>,
    pub opponent: Option<i64>,
}

/// Uma página dos mapas confirmados de `user` que passam no filtro, do mais
/// recente (a confirmação, depois o id), com o efeito no rating dos que
/// contaram (o cache do rating já recalculado por quem chama) e quantos o
/// filtro acha ao todo.
pub async fn games_page(
    conn: &mut PgConnection,
    tenant: TenantId,
    user: i64,
    filter: &GamesFilter<'_>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<MemberGame>, i64), ApiError> {
    let rows = sqlx::query!(
        r#"select g.id, t.slug, t.locale, t.name,
                  coalesce((select json_agg(json_build_object('locale', tr.locale, 'name', tr.name)
                                            order by tr.locale)
                            from tournament_translations tr where tr.tournament_id = t.id),
                           '[]') as "translations!: Json<Vec<NameTranslation>>",
                  g.stage, g.match_key,
                  (s.format = 'elimination' and s.bracket is not distinct from 'single')
                      as "single!",
                  g.number, mp.slug as "map_slug?", mp.name as "map_name?",
                  case p.side when 'a' then g.side_b else g.side_a end as "opponent!",
                  g.winner = case p.side when 'a' then g.side_a else g.side_b end as "won!",
                  case p.side when 'a' then g.a_frags else g.b_frags end as frags,
                  case p.side when 'a' then g.b_frags else g.a_frags end as opponent_frags,
                  h.delta as "delta?", h.rating_after as "rating_after?",
                  g.confirmed_at as "at!",
                  count(*) over () as "total!"
           from tournaments t
           join match_game_players p on p.tournament_id = t.id and p.user_id = $2
           join match_games g on g.id = p.game_id
           join tournament_stages s on s.tournament_id = t.id and s.position = g.stage
           left join maps mp on mp.id = g.map_id
           left join rating_history h
                  on h.tenant_id = t.tenant_id and h.user_id = p.user_id and h.game_id = g.id
           where t.tenant_id = $1 and g.status = 'confirmed'
             and ($3::text is null or t.slug = $3)
             and ($4::text is null or mp.slug = $4)
             and ($5::bigint is null or exists (
                 select 1 from match_game_players o
                 where o.game_id = g.id and o.user_id = $5 and o.side <> p.side))
           order by g.confirmed_at desc, g.id desc
           limit $6 offset $7"#,
        tenant.get(),
        user,
        filter.tournament,
        filter.map,
        filter.opponent,
        limit,
        offset
    )
    .fetch_all(&mut *conn)
    .await?;
    // A página além do fim não traz linha: a contagem sai à parte.
    let total = match rows.first() {
        Some(r) => r.total,
        None if offset > 0 => count_games(conn, tenant, user, filter).await?,
        None => 0,
    };
    let ids: Vec<i64> = rows.iter().map(|r| r.opponent).collect();
    let opponents = teams::by_ids(conn, tenant, &ids).await?;
    let games = rows
        .into_iter()
        .map(|r| {
            let opponent = opponents
                .iter()
                .find(|t| t.id == r.opponent)
                .ok_or_else(ApiError::internal)?;
            Ok(MemberGame {
                tournament: TournamentLabel {
                    slug: r.slug,
                    locale: Locale::from_db(&r.locale)?,
                    name: r.name,
                    translations: r.translations.0,
                },
                match_key: MatchKey {
                    stage: r.stage,
                    local: r.match_key,
                }
                .public(),
                single: r.single,
                game: r.number,
                map: r
                    .map_slug
                    .zip(r.map_name)
                    .map(|(slug, name)| MapLabel { slug, name }),
                opponent: opponent.view(),
                won: r.won,
                frags: r.frags,
                opponent_frags: r.opponent_frags,
                rating: r
                    .delta
                    .zip(r.rating_after)
                    .map(|(delta, rating_after)| GameRating {
                        delta,
                        rating_after,
                    }),
                at: r.at,
            })
        })
        .collect::<Result<_, ApiError>>()?;
    Ok((games, total))
}

/// Quantos mapas de `user` passam no filtro.
async fn count_games(
    conn: &mut PgConnection,
    tenant: TenantId,
    user: i64,
    filter: &GamesFilter<'_>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select count(*) as "count!"
           from tournaments t
           join match_game_players p on p.tournament_id = t.id and p.user_id = $2
           join match_games g on g.id = p.game_id
           left join maps mp on mp.id = g.map_id
           where t.tenant_id = $1 and g.status = 'confirmed'
             and ($3::text is null or t.slug = $3)
             and ($4::text is null or mp.slug = $4)
             and ($5::bigint is null or exists (
                 select 1 from match_game_players o
                 where o.game_id = g.id and o.user_id = $5 and o.side <> p.side))"#,
        tenant.get(),
        user,
        filter.tournament,
        filter.map,
        filter.opponent
    )
    .fetch_one(conn)
    .await
}

#[derive(Deserialize, Debug)]
struct PersonRow {
    user_id: i64,
    nickname: String,
    avatar_url: Option<String>,
    active: bool,
}

#[derive(Deserialize, Debug)]
struct TournamentRow {
    slug: String,
    locale: Locale,
    name: String,
    translations: Vec<NameTranslation>,
}

/// As opções dos filtros de mapas de `user`, numa ida só: os torneios em que
/// ele tem mapa confirmado (do mais recente), as arenas (por nome) e quem
/// esteve do outro lado (por apelido).
pub async fn game_filters(
    conn: &mut PgConnection,
    tenant: TenantId,
    user: i64,
) -> Result<GameFilters, sqlx::Error> {
    let r = sqlx::query!(
        r#"with mine as (
               select t.id as tournament_id, t.tenant_id, g.id as game_id, g.map_id, p.side,
                      g.confirmed_at
               from tournaments t
               join match_game_players p on p.tournament_id = t.id and p.user_id = $2
               join match_games g on g.id = p.game_id
               where t.tenant_id = $1 and g.status = 'confirmed'
           )
           select
               coalesce((select json_agg(x order by x.last_at desc, x.slug) from (
                   select t.slug, t.locale, t.name, max(m.confirmed_at) as last_at,
                          coalesce((select json_agg(json_build_object('locale', tr.locale,
                                                                      'name', tr.name)
                                                    order by tr.locale)
                                    from tournament_translations tr
                                    where tr.tournament_id = t.id), '[]'::json) as translations
                   from mine m join tournaments t on t.id = m.tournament_id
                   group by t.id) x), '[]') as "tournaments!: Json<Vec<TournamentRow>>",
               coalesce((select json_agg(x order by x.name, x.slug) from (
                   select distinct mp.slug, mp.name
                   from mine m join maps mp on mp.id = m.map_id) x), '[]')
                   as "maps!: Json<Vec<MapLabel>>",
               coalesce((select json_agg(x order by lower(x.nickname), x.user_id) from (
                   select distinct o.user_id, tm.nickname,
                          coalesce(u.avatar_key, u.avatar_url) as avatar_url,
                          tm.left_at is null as active
                   from mine m
                   join match_game_players o on o.game_id = m.game_id and o.side <> m.side
                   join tenant_members tm on tm.tenant_id = m.tenant_id and tm.user_id = o.user_id
                   join users u on u.id = o.user_id) x), '[]')
                   as "opponents!: Json<Vec<PersonRow>>""#,
        tenant.get(),
        user
    )
    .fetch_one(conn)
    .await?;
    Ok(GameFilters {
        tournaments: r
            .tournaments
            .0
            .into_iter()
            .map(|t| TournamentLabel {
                slug: t.slug,
                locale: t.locale,
                name: t.name,
                translations: t.translations,
            })
            .collect(),
        maps: r.maps.0,
        opponents: r
            .opponents
            .0
            .into_iter()
            .map(|p| PersonLabel {
                user_id: p.user_id,
                nickname: p.nickname,
                avatar_url: resolve_avatar(p.avatar_url),
                active: p.active,
            })
            .collect(),
    })
}
