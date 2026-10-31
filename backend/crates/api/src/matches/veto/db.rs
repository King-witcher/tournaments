//! SQL do veto: o histórico de eventos (`match_veto_events`), os campeões do
//! torneio e os torneios com veto aberto, para o relógio.

use chrono::{DateTime, Utc};
use engine::core::{ChampionId, MapId, ParticipantId};
use sqlx::{PgConnection, PgPool};

use super::state::{Kind, VetoRow};
use crate::{
    matches::{db::MatchRef, result::Side},
    shared::ApiError,
    tenants::TenantId,
    tournaments::TournamentId,
};

/// Um campeão do jogo; os ativos são o pool de todo torneio.
#[derive(Clone, Debug)]
pub struct Champion {
    pub id: ChampionId,
    pub slug: String,
    pub name: String,
    pub active: bool,
}

/// Todos os campeões (os inativos aparecem em vetos antigos), na ordem dos
/// ids: a ordem das opções e do sorteio do tempo esgotado, que não muda.
pub async fn champions(conn: &mut PgConnection) -> Result<Vec<Champion>, sqlx::Error> {
    Ok(
        sqlx::query!("select id, slug, name, active from champions order by id")
            .fetch_all(conn)
            .await?
            .into_iter()
            .map(|r| Champion {
                id: ChampionId(r.id),
                slug: r.slug,
                name: r.name,
                active: r.active,
            })
            .collect(),
    )
}

/// Os eventos de veto da fase `stage`, por partida e na ordem, com o apelido
/// de quem agiu.
pub async fn events(
    conn: &mut PgConnection,
    tournament: TournamentId,
    stage: i16,
) -> Result<Vec<VetoRow>, ApiError> {
    let rows = sqlx::query!(
        r#"select e.id, e.match_key, e.side_a, e.side_b, e.game, e.kind, e.side, e.map_id,
                  e.played_map_id, e.champion_id, e.player_id, e.pick_game, e.gave_way, e.seed,
                  e.timeout, e.actor,
                  coalesce(m.nickname, u.display_name) as actor_name,
                  pm.nickname as "player_name?", e.created_at
           from match_veto_events e
           left join users u on u.id = e.actor
           left join tenant_members m on m.tenant_id = e.tenant_id and m.user_id = e.actor
           left join tenant_members pm on pm.tenant_id = e.tenant_id and pm.user_id = e.player_id
           where e.tournament_id = $1 and e.stage = $2
           order by e.match_key, e.id"#,
        tournament.get(),
        stage
    )
    .fetch_all(conn)
    .await?;
    rows.into_iter()
        .map(|r| {
            Ok(VetoRow {
                id: r.id,
                match_id: crate::matches::key::parse(&r.match_key)
                    .ok_or_else(|| crate::matches::result::unknown("match_key", &r.match_key))?,
                sides: (ParticipantId(r.side_a), ParticipantId(r.side_b)),
                game: r.game,
                kind: Kind::from_db(&r.kind)?,
                side: r.side.as_deref().map(Side::from_db).transpose()?,
                map: r.map_id.map(MapId),
                played_map: r.played_map_id.map(MapId),
                champion: r.champion_id.map(ChampionId),
                player: r.player_id,
                player_name: r.player_name,
                pick_game: r.pick_game,
                gave_way: r.gave_way,
                seed: r.seed,
                timeout: r.timeout,
                actor: r.actor,
                actor_name: r.actor_name,
                created_at: r.created_at,
            })
        })
        .collect()
}

/// Um evento a gravar.
pub struct NewVetoEvent {
    pub kind: Kind,
    pub game: Option<i16>,
    pub side: Option<Side>,
    pub map: Option<MapId>,
    pub played_map: Option<MapId>,
    pub champion: Option<ChampionId>,
    /// TDM: o jogador da janela simultânea.
    pub player: Option<i64>,
    /// Pick de campeão: a partida a que ele se destina.
    pub pick_game: Option<i16>,
    /// TDM: o pick veio com alguma regra cedida.
    pub gave_way: bool,
    pub seed: Option<i64>,
    pub timeout: bool,
    pub actor: Option<i64>,
    /// Quando; `None`: agora.
    pub at: Option<DateTime<Utc>>,
}

impl NewVetoEvent {
    pub const fn new(
        kind: Kind,
        game: Option<i16>,
        side: Option<Side>,
        actor: Option<i64>,
    ) -> Self {
        Self {
            kind,
            game,
            side,
            map: None,
            played_map: None,
            champion: None,
            player: None,
            pick_game: None,
            gave_way: false,
            seed: None,
            timeout: false,
            actor,
            at: None,
        }
    }
}

/// Grava um evento do veto da partida `at`, entre `sides`.
pub async fn insert(
    conn: &mut PgConnection,
    at: &MatchRef<'_>,
    sides: (ParticipantId, ParticipantId),
    event: &NewVetoEvent,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"insert into match_veto_events
               (tenant_id, tournament_id, stage, match_key, side_a, side_b, kind, game, side,
                map_id, played_map_id, champion_id, seed, timeout, actor, created_at, player_id,
                pick_game, gave_way)
           values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15,
                   coalesce($16, now()), $17, $18, $19)"#,
        at.tenant.get(),
        at.tournament.get(),
        at.stage,
        at.key,
        sides.0.0,
        sides.1.0,
        event.kind.as_str(),
        event.game,
        event.side.map(Side::as_str),
        event.map.map(|m| m.0),
        event.played_map.map(|m| m.0),
        event.champion.map(|c| c.0),
        event.seed,
        event.timeout,
        event.actor,
        event.at,
        event.player,
        event.pick_game,
        event.gave_way
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Apaga o veto da partida `at` (ela passou a ter outros lados).
pub async fn remove(conn: &mut PgConnection, at: &MatchRef<'_>) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "delete from match_veto_events
         where tournament_id = $1 and stage = $2 and match_key = $3",
        at.tournament.get(),
        at.stage,
        at.key
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Torneios em andamento com veto começado numa partida ainda sem série
/// decidida: os que o relógio olha.
pub async fn open_tournaments(db: &PgPool) -> Result<Vec<(TenantId, String)>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"select t.tenant_id, t.slug from tournaments t
           where t.status = 'in_progress'
             and exists(select 1 from match_veto_events e
                        where e.tournament_id = t.id and e.kind = 'begin'
                          and not exists(select 1 from match_results r
                                         where r.tournament_id = e.tournament_id
                                           and r.stage = e.stage
                                           and r.match_key = e.match_key))
           order by t.id"#
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|r| (TenantId::new(r.tenant_id), r.slug))
    .collect())
}

/// TDM: quem jogou cada partida enviada da fase `stage` (a escalação gravada
/// no envio, `match_game_players`), por partida e lado: `(série, partida,
/// lado, jogador)`, na ordem.
pub async fn lineups(
    conn: &mut PgConnection,
    tournament: TournamentId,
    stage: i16,
) -> Result<Vec<(engine::core::MatchId, i16, Side, i64)>, ApiError> {
    let rows = sqlx::query!(
        r#"select g.match_key, g.number, gp.side, gp.user_id
           from match_game_players gp
           join match_games g on g.id = gp.game_id
           where g.tournament_id = $1 and g.stage = $2
           order by g.match_key, g.number, gp.side, gp.user_id"#,
        tournament.get(),
        stage
    )
    .fetch_all(conn)
    .await?;
    rows.into_iter()
        .map(|r| {
            let id = crate::matches::key::parse(&r.match_key)
                .ok_or_else(|| crate::matches::result::unknown("match_key", &r.match_key))?;
            Ok((id, r.number, Side::from_db(&r.side)?, r.user_id))
        })
        .collect()
}
