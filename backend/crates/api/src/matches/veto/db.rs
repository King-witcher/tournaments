//! SQL do veto: gravar e apagar o histórico de eventos (`match_veto_events`)
//! e os torneios com veto aberto, com o carimbo do histórico, para o relógio.
//! Os eventos, as escalações e os campeões são lidos com a fase
//! (`db::stage_rows`).

use chrono::{DateTime, Utc};
use engine::core::{ChampionId, MapId, ParticipantId};
use sqlx::{PgConnection, PgPool};

use super::state::Kind;
use crate::{
    matches::{db::MatchRef, result::Side},
    tenants::TenantId,
};

/// Um campeão do jogo; os ativos são o pool de todo torneio.
#[derive(Clone, Debug)]
pub struct Champion {
    pub id: ChampionId,
    pub slug: String,
    pub name: String,
    pub active: bool,
}

/// Um evento a gravar.
pub struct NewVetoEvent {
    pub kind: Kind,
    pub game: Option<i16>,
    pub side: Option<Side>,
    pub map: Option<MapId>,
    pub played_map: Option<MapId>,
    pub champion: Option<ChampionId>,
    /// No pick por jogador, o jogador-alvo: de quem é o campeão. `actor` é quem
    /// clicou; nulo no sorteado.
    pub player: Option<i64>,
    /// Pick de campeão: a partida a que ele se destina.
    pub pick_game: Option<i16>,
    /// No pick por jogador: veio com alguma regra cedida.
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

/// Um torneio que o relógio olha.
pub struct OpenTournament {
    /// O id interno, só para o relógio reconhecer o torneio entre as passadas.
    pub id: i64,
    pub tenant: TenantId,
    pub tenant_slug: String,
    pub slug: String,
    pub stamp: Stamp,
}

/// O carimbo do histórico do veto de um torneio (todas as fases): muda a cada
/// evento gravado ou apagado (o histórico só cresce, e apagar o veto de uma
/// série tira linhas) e com a hora de um evento mexida à mão (os testes
/// envelhecem o veto assim).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stamp {
    pub events: i64,
    pub last_id: i64,
    pub last_at: DateTime<Utc>,
}

/// Torneios em andamento com veto começado numa partida ainda sem série
/// decidida: os que o relógio olha, com o carimbo do histórico de cada um.
pub async fn open_tournaments(db: &PgPool) -> Result<Vec<OpenTournament>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"select t.id, t.tenant_id, n.slug as tenant_slug, t.slug,
                  s.events as "events!", s.last_id as "last_id!", s.last_at as "last_at!"
           from tournaments t
           join tenants n on n.id = t.tenant_id
           cross join lateral (select count(*) as events, max(e.id) as last_id,
                                      max(e.created_at) as last_at
                               from match_veto_events e
                               where e.tournament_id = t.id) s
           where t.status = 'in_progress'
             and exists(select 1 from match_veto_events e
                        where e.tournament_id = t.id and e.kind in ('begin', 'ready')
                          and not exists(select 1 from match_results r
                                         where r.tournament_id = e.tournament_id
                                           and r.stage = e.stage
                                           and r.match_key = e.match_key))
           order by t.id"#
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|r| OpenTournament {
        id: r.id,
        tenant: TenantId::new(r.tenant_id),
        tenant_slug: r.tenant_slug,
        slug: r.slug,
        stamp: Stamp {
            events: r.events,
            last_id: r.last_id,
            last_at: r.last_at,
        },
    })
    .collect())
}
