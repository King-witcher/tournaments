//! O veto como fica gravado e o estado que o `engine` refaz dele
//! (`veto::Veto::replay`): `VetoLog` serve à série de um torneio
//! (`match_veto_events`, `MatchVeto`) e à sala de veto (`veto_rooms`). O
//! "pronto" de cada lado, o pedido de desfazer e o relógio do passo são da API:
//! saem das linhas.

use chrono::{DateTime, Utc};
use engine::{
    core::{ChampionId, MapId, MatchId, ParticipantId, PlayerId},
    veto::{self as ev, Action, Event, Rosters, Turn, Veto},
};

use crate::{
    matches::result::{Side, unknown},
    shared::ApiError,
};

/// Um evento gravado de um veto, sem dizer de quem é. `actor` e `player` são
/// quem agiu: na série, a conta; na sala, o lugar.
#[derive(Clone, Debug)]
pub struct VetoRow {
    pub id: i64,
    pub game: Option<i16>,
    pub kind: Kind,
    pub side: Option<Side>,
    pub map: Option<MapId>,
    pub played_map: Option<MapId>,
    pub champion: Option<ChampionId>,
    /// TDM: na janela simultânea, quem escolheu o campeão (e o apelido).
    pub player: Option<i64>,
    pub player_name: Option<String>,
    /// Pick de campeão: a partida a que ele se destina.
    pub pick_game: Option<i16>,
    /// TDM: o pick veio com alguma regra cedida.
    pub gave_way: bool,
    pub seed: Option<i64>,
    pub timeout: bool,
    pub actor: Option<i64>,
    pub actor_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// `match_veto_events.kind`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Ready,
    Begin,
    Game,
    Ban,
    Pick,
    Vote,
    Undo,
    UndoRequest,
    UndoDeclined,
}

impl Kind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Begin => "begin",
            Self::Game => "game",
            Self::Ban => "ban",
            Self::Pick => "pick",
            Self::Vote => "vote",
            Self::Undo => "undo",
            Self::UndoRequest => "undo_request",
            Self::UndoDeclined => "undo_declined",
        }
    }

    pub fn from_db(value: &str) -> Result<Self, ApiError> {
        use Kind::*;
        [
            Ready,
            Begin,
            Game,
            Ban,
            Pick,
            Vote,
            Undo,
            UndoRequest,
            UndoDeclined,
        ]
        .into_iter()
        .find(|k| k.as_str() == value)
        .ok_or_else(|| unknown("kind", value))
    }

    pub const fn of(action: ev::Action) -> Self {
        match action {
            ev::Action::Ban => Self::Ban,
            ev::Action::Pick | ev::Action::TeamPick => Self::Pick,
            ev::Action::Vote => Self::Vote,
        }
    }

    /// Reinicia o relógio do passo.
    const fn clock(self) -> bool {
        matches!(
            self,
            Self::Begin | Self::Game | Self::Ban | Self::Pick | Self::Vote | Self::Undo
        )
    }
}

impl VetoRow {
    /// Reinicia o relógio do passo: a janela simultânea dos times não
    /// reinicia (a janela é uma só para todos).
    const fn resets_clock(&self) -> bool {
        self.kind.clock() && self.player.is_none()
    }

    /// É evento do `engine` (os que só a API usa: pronto, pedido e recusa de
    /// desfazer).
    const fn is_event(&self) -> bool {
        !matches!(
            self.kind,
            Kind::Ready | Kind::UndoRequest | Kind::UndoDeclined
        )
    }

    /// O evento do `engine`; `None` nos que só a API usa (`is_event`).
    fn event(&self) -> Result<Option<Event>, ApiError> {
        let side = || {
            self.side
                .map(ev_side)
                .ok_or_else(|| unknown("side", "null"))
        };
        let game = || {
            self.game
                .and_then(|g| u8::try_from(g).ok())
                .ok_or_else(|| unknown("game", "null"))
        };
        let action = match self.kind {
            Kind::Ready | Kind::UndoRequest | Kind::UndoDeclined => return Ok(None),
            Kind::Undo => return Ok(Some(Event::Undo)),
            Kind::Begin => {
                return Ok(Some(Event::Begin {
                    // A seed é o u64 gravado bit a bit.
                    seed: self.seed.ok_or_else(|| unknown("seed", "null"))? as u64,
                    first: side()?,
                }));
            }
            Kind::Game => {
                return Ok(Some(Event::Game {
                    game: game()?,
                    previous: self.played_map,
                }));
            }
            Kind::Ban => ev::Action::Ban,
            Kind::Pick => ev::Action::Pick,
            Kind::Vote => ev::Action::Vote,
        };
        let choice = match (self.map, self.champion) {
            (Some(m), None) => ev::Choice::Map(m),
            (None, Some(c)) => ev::Choice::Champion(c),
            _ => return Err(unknown("choice", "?")),
        };
        if let (Some(player), ev::Choice::Champion(champion)) = (self.player, choice) {
            return Ok(Some(Event::TeamPick {
                side: side()?,
                player: PlayerId(player),
                game: self
                    .pick_game
                    .and_then(|g| u8::try_from(g).ok())
                    .ok_or_else(|| unknown("pick_game", "null"))?,
                gave_way: self.gave_way,
                champion,
            }));
        }
        Ok(Some(Event::Step {
            side: side()?,
            action,
            choice,
        }))
    }
}

pub const fn ev_side(side: Side) -> engine::series::Side {
    match side {
        Side::A => engine::series::Side::A,
        Side::B => engine::series::Side::B,
    }
}

pub const fn api_side(side: engine::series::Side) -> Side {
    match side {
        engine::series::Side::A => Side::A,
        engine::series::Side::B => Side::B,
    }
}

pub const fn other(side: Side) -> Side {
    match side {
        Side::A => Side::B,
        Side::B => Side::A,
    }
}

pub const fn slot(side: Side) -> usize {
    match side {
        Side::A => 0,
        Side::B => 1,
    }
}

/// Um evento gravado do veto de uma série de torneio.
#[derive(Clone, Debug)]
pub struct MatchVetoRow {
    pub match_id: MatchId,
    pub sides: (ParticipantId, ParticipantId),
    pub row: VetoRow,
}

/// O veto de uma série de torneio. Lê como o `VetoLog` dela (`Deref`).
#[derive(Clone, Debug)]
pub struct MatchVeto {
    pub match_id: MatchId,
    pub sides: (ParticipantId, ParticipantId),
    pub log: VetoLog,
}

impl std::ops::Deref for MatchVeto {
    type Target = VetoLog;

    fn deref(&self) -> &VetoLog {
        &self.log
    }
}

/// Um veto: as linhas e o estado refeito delas.
#[derive(Clone, Debug)]
pub struct VetoLog {
    pub rows: Vec<VetoRow>,
    pub state: Veto,
}

impl VetoLog {
    /// Refaz o veto de `rows` (na ordem), com o que o histórico não diz nos
    /// times (`rosters`: quem escolhe campeão de cada lado agora, quantos mapas
    /// foram enviados e se a série acabou). Erro interno se o histórico não
    /// fecha ou o elenco não vale.
    pub fn replay(
        rules: ev::VetoRules,
        best_of: u8,
        rosters: Rosters,
        rows: Vec<VetoRow>,
    ) -> Result<Self, ApiError> {
        let mut events = Vec::with_capacity(rows.len());
        for row in &rows {
            if let Some(e) = row.event()? {
                events.push(e);
            }
        }
        let mut state = Veto::replay(rules, best_of, events).map_err(|e| {
            tracing::error!(%e, "histórico do veto não fecha");
            ApiError::internal()
        })?;
        state.set_rosters(rosters).map_err(|e| {
            tracing::error!(%e, "elenco do veto recusado");
            ApiError::internal()
        })?;
        Ok(Self { rows, state })
    }

    /// Quem já disse que está pronto (`[a, b]`).
    pub fn ready(&self) -> [bool; 2] {
        let mut ready = [false, false];
        for r in self.rows.iter().filter(|r| r.kind == Kind::Ready) {
            if let Some(side) = r.side {
                ready[slot(side)] = true;
            }
        }
        ready
    }

    /// O lado com um pedido de desfazer à espera da resposta do outro: o
    /// último pedido, se nada aconteceu no veto depois dele.
    pub fn undo_request(&self) -> Option<Side> {
        let mut pending = None;
        for r in &self.rows {
            match r.kind {
                Kind::UndoRequest => pending = r.side,
                Kind::Ready => {}
                _ => pending = None,
            }
        }
        pending.filter(|_| self.state.can_undo())
    }

    /// A vez de agora: o id da linha do evento que a abriu
    /// (`Veto::turn_opened`); `None` antes do começo. Muda a cada passo
    /// feito, sorteado ou desfeito e a cada partida nova; o voto que espera o
    /// do outro e as escolhas da janela simultânea não a mudam.
    pub fn step(&self) -> Option<i64> {
        let n = usize::try_from(self.state.turn_opened()).ok()?;
        let row = self
            .rows
            .iter()
            .filter(|r| r.is_event())
            .nth(n.checked_sub(1)?)?;
        Some(row.id)
    }

    /// Quando começou o passo em andamento (ou a escolha simultânea dos
    /// times): o último evento que reinicia o relógio.
    pub fn clock(&self) -> Option<DateTime<Utc>> {
        self.rows
            .iter()
            .rev()
            .find(|r| r.resets_clock())
            .map(|r| r.created_at)
    }

    /// Até quando o passo em andamento pode ser feito, com `seconds` por
    /// passo e `team_seconds` para a escolha simultânea dos times; `None`
    /// sem passo esperando.
    pub fn deadline(&self, seconds: i32, team_seconds: i32) -> Option<DateTime<Utc>> {
        let seconds = match self.state.turn() {
            Turn::Act {
                action: Action::TeamPick,
                ..
            } => team_seconds,
            Turn::Act { .. } => seconds,
            _ => return None,
        };
        Some(self.clock()? + chrono::Duration::seconds(i64::from(seconds)))
    }
}
