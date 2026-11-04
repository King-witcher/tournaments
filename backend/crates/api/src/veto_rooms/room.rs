//! A sala carregada: a linha, os lugares e o veto refeito do histórico, com o
//! que ela grava (eventos, o próximo prazo, o nome de um lugar).

use chrono::{DateTime, Utc};
use engine::{
    champions::{ChampionRules, NoRepeatRule, NoRepeatScope, NoRepeatWho as EngineWho},
    core::{ChampionId, MapId, PlayerId},
    veto::{self as ev, Rosters},
};
use sqlx::PgConnection;

use super::{
    db::{self, RoomRow},
    types::{VetoRoomActors, VetoRoomRole, VetoRoomStart},
};
use crate::{
    matches::{
        Side,
        veto::{Champion, Ctx, GRACE, NewVetoEvent, Readiness, VetoLog, VetoRow, due_draws},
    },
    shared::ApiError,
    tournaments::{NoRepeatWho, PickOrder, PoolMap, VetoOrder},
};

pub(super) const NOT_FOUND: ApiError = ApiError::not_found("veto_room.not_found");

/// Um lugar.
pub(super) struct Seat {
    pub id: i64,
    pub side: Side,
    pub position: i16,
    /// `None`: o lugar sem link, na sala centrada no capitão.
    pub token: Option<String>,
    pub name: Option<String>,
}

/// A sala do link com que foi aberta.
pub(super) struct Room {
    pub id: i64,
    pub manage_token: String,
    pub spectator_token: String,
    pub team_size: i16,
    pub best_of: i16,
    pub bans_per_series: i16,
    pub order: VetoOrder,
    pub no_repeat: Option<NoRepeatWho>,
    pub who_starts: VetoRoomStart,
    pub who_acts: VetoRoomActors,
    pub rules: ev::VetoRules,
    pub step_seconds: i32,
    pub team_pick_seconds: i32,
    pub maps: Vec<PoolMap>,
    pub champions: Vec<Champion>,
    pub seats: Vec<Seat>,
    pub expires_at: DateTime<Utc>,
    due_at: Option<DateTime<Utc>>,
    pub role: VetoRoomRole,
    /// O lugar deste link, se é de jogador.
    seat: Option<i64>,
    pub log: VetoLog,
    /// Esta transação mudou o que a sala mostra.
    pub changed: bool,
}

impl Room {
    /// A sala do link `token`. 404 `veto_room.not_found` (também vencida).
    pub async fn read(conn: &mut PgConnection, token: &str) -> Result<Self, ApiError> {
        let row = db::find(&mut *conn, token).await?.ok_or(NOT_FOUND)?;
        Self::of(conn, row).await
    }

    /// `read` com a sala travada até o fim da transação.
    pub async fn lock(conn: &mut PgConnection, token: &str) -> Result<Self, ApiError> {
        let row = db::lock(&mut *conn, token).await?.ok_or(NOT_FOUND)?;
        Self::of(conn, row).await
    }

    async fn of(conn: &mut PgConnection, row: RoomRow) -> Result<Self, ApiError> {
        let rows = db::events(conn, row.id).await?;
        let seats: Vec<Seat> = row
            .seats
            .0
            .into_iter()
            .map(|s| Seat {
                id: s.id,
                side: s.side,
                position: s.position,
                token: s.token,
                name: s.name,
            })
            .collect();
        let maps: Vec<PoolMap> = row
            .maps
            .0
            .into_iter()
            .map(|m| PoolMap {
                id: MapId(m.id),
                slug: m.slug,
                name: m.name,
            })
            .collect();
        let no_repeat = match row.no_repeat.as_deref() {
            None => None,
            Some("player") => Some(NoRepeatWho::Player),
            Some("team") => Some(NoRepeatWho::Team),
            Some(other) => {
                tracing::error!(
                    no_repeat = other,
                    "regra de repetição desconhecida no banco"
                );
                return Err(ApiError::internal());
            }
        };
        let order = VetoOrder::from_db(&row.veto_order)?;
        let rules = ev::VetoRules {
            map_mode: if row.map_mode == "off" {
                ev::MapMode::Off
            } else {
                ev::MapMode::Veto
            },
            order: order.into(),
            pool: maps.iter().map(|m| m.id).collect(),
            champion_phase: row.champion_phase,
            pick_order: PickOrder::from_db(&row.pick_order)?.into(),
            team_size: u8::try_from(row.team_size).unwrap_or(1),
            champions: ChampionRules {
                bans_per_series: u32::try_from(row.bans_per_series).unwrap_or(0),
                mirror: row.mirror,
                same_team: row.same_team,
                no_repeat: no_repeat.map(|who| NoRepeatRule {
                    who: match who {
                        NoRepeatWho::Player => EngineWho::Player,
                        NoRepeatWho::Team => EngineWho::Team,
                    },
                    scope: NoRepeatScope::Series,
                }),
            },
        };
        let role = match row.kind.as_str() {
            "manager" => VetoRoomRole::Manager,
            "spectator" => VetoRoomRole::Spectator,
            _ => VetoRoomRole::Player,
        };
        let mut rows = rows;
        fill_names(&mut rows, &seats);
        let log = VetoLog::replay(
            rules.clone(),
            u8::try_from(row.best_of).unwrap_or(1),
            rosters(&seats),
            rows,
        )
        .inspect_err(|_| tracing::error!(room = row.id, "o veto desta sala não fecha"))?;
        Ok(Self {
            id: row.id,
            manage_token: row.manage_token,
            spectator_token: row.spectator_token,
            team_size: row.team_size,
            best_of: row.best_of,
            bans_per_series: row.bans_per_series,
            order,
            no_repeat,
            who_starts: VetoRoomStart::from_db(&row.who_starts),
            who_acts: VetoRoomActors::from_db(&row.team_actors),
            rules,
            step_seconds: row.step_seconds,
            team_pick_seconds: row.team_pick_seconds,
            maps,
            champions: row
                .champions
                .0
                .into_iter()
                .map(|c| Champion {
                    id: ChampionId(c.id),
                    slug: c.slug,
                    name: c.name,
                    active: c.active,
                })
                .collect(),
            seats,
            expires_at: row.expires_at,
            due_at: row.due_at,
            role,
            seat: row.seat_id,
            log,
            changed: false,
        })
    }

    /// O sorteio já venceu em `now`.
    pub fn is_due(&self, now: DateTime<Utc>) -> bool {
        self.due_at.is_some_and(|due| due <= now)
    }

    /// Melhor de N.
    pub fn best_of(&self) -> u8 {
        u8::try_from(self.best_of).unwrap_or(1)
    }

    /// O lugar deste link.
    pub fn seat(&self) -> Option<&Seat> {
        self.seats.iter().find(|s| Some(s.id) == self.seat)
    }

    /// O lugar deste link, pronto para agir. 403 `veto_room.not_player` (link
    /// de gestão ou de espectador), 409 `veto_room.not_joined`.
    pub fn acting_seat(&self) -> Result<(i64, Side), ApiError> {
        let seat = self
            .seat()
            .ok_or(ApiError::forbidden("veto_room.not_player"))?;
        if seat.name.is_none() {
            return Err(ApiError::conflict("veto_room.not_joined"));
        }
        Ok((seat.id, seat.side))
    }

    /// Quem falta marcar "Pronto": todo lugar com link. Na sala centrada no
    /// capitão, são os dois capitães; nas outras, todos os lugares, com gente ou não
    /// (a gestão começa sem esperar quem não apareceu).
    pub fn readiness(&self) -> Readiness {
        let of = |side: Side| {
            self.seats
                .iter()
                .filter(|s| s.side == side && s.token.is_some())
                .map(|s| s.id)
                .collect()
        };
        Readiness::of(Some(&self.log), [of(Side::A), of(Side::B)])
    }

    /// Grava `events`, refaz o veto com eles e acerta o próximo prazo.
    pub async fn record(
        &mut self,
        conn: &mut PgConnection,
        events: Vec<NewVetoEvent>,
    ) -> Result<(), ApiError> {
        let mut rows = std::mem::take(&mut self.log.rows);
        for event in &events {
            let (id, created_at) = db::insert_event(&mut *conn, self.id, event).await?;
            rows.push(VetoRow {
                id,
                game: event.game,
                kind: event.kind,
                side: event.side,
                map: event.map,
                played_map: None,
                champion: event.champion,
                player: event.player,
                player_name: None,
                pick_game: event.pick_game,
                gave_way: event.gave_way,
                seed: event.seed,
                timeout: event.timeout,
                actor: event.actor,
                actor_name: None,
                created_at,
            });
        }
        fill_names(&mut rows, &self.seats);
        self.log = VetoLog::replay(
            self.rules.clone(),
            self.best_of(),
            rosters(&self.seats),
            rows,
        )?;
        self.changed = true;
        let due = self
            .log
            .deadline(self.step_seconds, self.team_pick_seconds)
            .map(|deadline| deadline + GRACE);
        if due != self.due_at {
            db::set_due(conn, self.id, due).await?;
            self.due_at = due;
        }
        Ok(())
    }

    /// Sorteia e grava os passos vencidos em `now`. Quem chama trava a sala.
    /// Devolve quantos gravou.
    pub async fn apply_due(
        &mut self,
        conn: &mut PgConnection,
        now: DateTime<Utc>,
    ) -> Result<usize, ApiError> {
        let Some(first) = self.due_at.filter(|due| *due <= now) else {
            return Ok(0);
        };
        let ctx = Ctx::alone(&self.champions);
        let mut state = self.log.state.clone();
        let rows = due_draws(
            &mut state,
            &ctx,
            first,
            now,
            self.step_seconds,
            self.team_pick_seconds,
        )?;
        let written = rows.len();
        if written == 0 {
            // Ninguém tem opção: sem isto, o relógio voltaria aqui para sempre.
            tracing::error!(room = self.id, "sala de veto sem opção para sortear");
            db::set_due(conn, self.id, None).await?;
            self.due_at = None;
            return Ok(0);
        }
        self.record(conn, rows).await?;
        Ok(written)
    }

    /// A sala começou: passa a valer `hours` a partir de agora.
    pub async fn extend(&mut self, conn: &mut PgConnection, hours: i32) -> Result<(), ApiError> {
        self.expires_at = db::extend(conn, self.id, hours).await?;
        Ok(())
    }

    /// Dá nome ao lugar `seat`.
    pub async fn set_name(
        &mut self,
        conn: &mut PgConnection,
        seat: i64,
        name: String,
    ) -> Result<(), ApiError> {
        db::set_name(conn, seat, &name).await?;
        if let Some(s) = self.seats.iter_mut().find(|s| s.id == seat) {
            s.name = Some(name);
        }
        fill_names(&mut self.log.rows, &self.seats);
        self.changed = true;
        Ok(())
    }
}

/// O elenco de cada lado: todos os lugares, com gente ou não. Quem não apareceu
/// fica para o sorteio do fim do prazo, como no TDM.
fn rosters(seats: &[Seat]) -> Rosters {
    let of = |side: Side| {
        seats
            .iter()
            .filter(|s| s.side == side)
            .map(|s| PlayerId(s.id))
            .collect()
    };
    Rosters {
        players: [of(Side::A), of(Side::B)],
        lineups: Vec::new(),
        finished: false,
    }
}

/// Os nomes de quem agiu em cada linha, dos lugares.
fn fill_names(rows: &mut [VetoRow], seats: &[Seat]) {
    let name = |id: Option<i64>| {
        let id = id?;
        seats.iter().find(|s| s.id == id)?.name.clone()
    };
    for row in rows {
        row.actor_name = name(row.actor);
        row.player_name = name(row.player);
    }
}
