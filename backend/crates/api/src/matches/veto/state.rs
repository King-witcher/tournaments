//! O veto como fica gravado e o estado que o `engine` refaz dele
//! (`veto::Veto::replay`): `VetoLog` serve à série de um torneio
//! (`match_veto_events`, `MatchVeto`) e à sala de veto (`veto_rooms`). O
//! "pronto" de cada jogador (e o desfazer dele), o pedido de desfazer e o relógio
//! do passo são da API: saem das linhas.

use chrono::{DateTime, Utc};
use engine::{
    core::{ChampionId, MapId, MatchId, ParticipantId, PlayerId},
    veto::{self as ev, Event, Rosters, Target, Turn, Veto},
};

use crate::{
    matches::result::{Side, unknown},
    shared::ApiError,
    tournaments::TeamAction,
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
    /// No pick por jogador, de quem é o campeão (e o apelido).
    pub player: Option<i64>,
    pub player_name: Option<String>,
    /// Pick de campeão: a partida a que ele se destina.
    pub pick_game: Option<i16>,
    /// No pick por jogador: veio com alguma regra cedida.
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
    /// Desfaz o "Pronto" de quem o marcou, antes do começo.
    Unready,
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
            Self::Unready => "unready",
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
            Unready,
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

    /// Só a API usa (não vai ao `engine` nem muda o veto): o pronto e o desfazer
    /// dele, o pedido de desfazer e a recusa.
    pub const fn api_only(self) -> bool {
        matches!(
            self,
            Self::Ready | Self::Unready | Self::UndoRequest | Self::UndoDeclined
        )
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
    /// Reinicia o relógio do passo: na janela simultânea, as escolhas não
    /// reiniciam (a janela é uma só para todos); na ordem alternada, cada pick
    /// por jogador é um passo.
    const fn resets_clock(&self, window: bool) -> bool {
        self.kind.clock() && !(window && self.player.is_some())
    }

    /// É evento do `engine` (não é dos que só a API usa, `Kind::api_only`).
    const fn is_event(&self) -> bool {
        !self.kind.api_only()
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
            Kind::Ready | Kind::Unready | Kind::UndoRequest | Kind::UndoDeclined => {
                return Ok(None);
            }
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

/// A regra que cobre a vez `turn`; `None` sem vez.
pub const fn turn_action(turn: &Turn) -> Option<TeamAction> {
    match turn {
        Turn::Act {
            target: Target::Map,
            ..
        } => Some(TeamAction::Veto),
        Turn::Act {
            action: ev::Action::Ban,
            ..
        } => Some(TeamAction::Ban),
        Turn::Act { .. } => Some(TeamAction::Pick),
        Turn::NotBegun | Turn::Done { .. } | Turn::RostersUnknown { .. } => None,
    }
}

/// Quem marcou "Pronto" (`(lado, autor)`), com `r`, se é um pronto ou o
/// desfazer dele: o último dos dois de cada um no lado diz se está marcado. O
/// pronto sem autor (conta excluída) fica.
fn mark(marks: &mut Vec<(Side, Option<i64>)>, r: &VetoRow) {
    let Some(side) = r.side else { return };
    match r.kind {
        Kind::Ready => marks.push((side, r.actor)),
        Kind::Unready => marks.retain(|m| *m != (side, r.actor)),
        _ => {}
    }
}

/// Quem ainda falta marcar "Pronto" e quem está marcado, por lado (`[a, b]`).
#[derive(Clone, Debug, Default)]
pub struct Readiness {
    pub waiting: [Vec<i64>; 2],
    /// De quem precisa marcar, quem está marcado (e pode desfazer).
    pub marked: [Vec<i64>; 2],
    /// O lado já esteve pronto em algum ponto do histórico, contado sobre quem
    /// precisa marcar agora: o aviso ao adversário sai só na primeira vez.
    pub been_ready: [bool; 2],
}

impl Readiness {
    /// De `required`, quem precisa marcar por lado (o elenco de agora, ou o
    /// capitão), sai quem está marcado no histórico `log`. Marcou e saiu do
    /// elenco: não conta. Saiu e voltou: continua marcado.
    pub fn of(log: Option<&VetoLog>, required: [Vec<i64>; 2]) -> Self {
        let mut marks = Vec::new();
        let mut been_ready = [false, false];
        for r in log.iter().flat_map(|l| &l.rows) {
            mark(&mut marks, r);
            if let (Kind::Ready, Some(side)) = (r.kind, r.side) {
                let all = required[slot(side)]
                    .iter()
                    .all(|w| marks.contains(&(side, Some(*w))));
                been_ready[slot(side)] |= all;
            }
        }
        let split = |side: Side, who: Vec<i64>| -> (Vec<i64>, Vec<i64>) {
            who.into_iter()
                .partition(|w| !marks.contains(&(side, Some(*w))))
        };
        let [a, b] = required;
        let (waiting_a, marked_a) = split(Side::A, a);
        let (waiting_b, marked_b) = split(Side::B, b);
        Self {
            waiting: [waiting_a, waiting_b],
            marked: [marked_a, marked_b],
            been_ready,
        }
    }

    /// O lado está pronto.
    pub fn side(&self, side: Side) -> bool {
        self.waiting[slot(side)].is_empty()
    }

    pub fn both(&self) -> bool {
        self.side(Side::A) && self.side(Side::B)
    }

    /// `who` ainda precisa marcar.
    pub fn awaits(&self, who: i64) -> bool {
        self.waiting.iter().flatten().any(|w| *w == who)
    }

    /// `who` está marcado e conta: pode desfazer.
    pub fn has_marked(&self, who: i64) -> bool {
        self.marked.iter().flatten().any(|w| *w == who)
    }

    /// Como fica depois de `who` marcar.
    pub fn without(mut self, who: i64) -> Self {
        for (waiting, marked) in self.waiting.iter_mut().zip(&mut self.marked) {
            if waiting.contains(&who) {
                waiting.retain(|w| *w != who);
                marked.push(who);
            }
        }
        self
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
    /// Refaz o veto de `rows` (na ordem), com o que o histórico não diz
    /// (`rosters`: quem escolhe campeão de cada lado agora, quantos mapas foram
    /// enviados e se a série acabou). Erro interno se o histórico não fecha ou o
    /// elenco não vale.
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

    /// Os lados em que alguém está com "Pronto" marcado (`[a, b]`): o que a tela
    /// mostra do veto já começado, que não pede mais nada.
    pub fn clicked(&self) -> [bool; 2] {
        let mut marks = Vec::new();
        for r in &self.rows {
            mark(&mut marks, r);
        }
        [Side::A, Side::B].map(|side| marks.iter().any(|(s, _)| *s == side))
    }

    /// A regra que cobre o último passo da partida em andamento, o que o desfazer
    /// tiraria; `None` sem passo.
    pub fn last_step(&self) -> Option<TeamAction> {
        let mut steps = Vec::new();
        for r in &self.rows {
            match r.kind {
                Kind::Ban | Kind::Pick | Kind::Vote => steps.push(r),
                Kind::Undo => {
                    steps.pop();
                }
                Kind::Begin | Kind::Game => steps.clear(),
                Kind::Ready | Kind::Unready | Kind::UndoRequest | Kind::UndoDeclined => {}
            }
        }
        let last = steps.last()?;
        Some(match (last.kind, last.map) {
            (_, Some(_)) => TeamAction::Veto,
            (Kind::Ban, None) => TeamAction::Ban,
            _ => TeamAction::Pick,
        })
    }

    /// O lado com um pedido de desfazer à espera da resposta do outro: o
    /// último pedido, se nada aconteceu no veto depois dele.
    pub fn undo_request(&self) -> Option<Side> {
        let mut pending = None;
        for r in &self.rows {
            match r.kind {
                Kind::UndoRequest => pending = r.side,
                Kind::Ready | Kind::Unready => {}
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

    /// Quando começou o passo em andamento (ou a janela simultânea): o último
    /// evento que reinicia o relógio.
    pub fn clock(&self) -> Option<DateTime<Utc>> {
        let window = self.state.rules().pick_order.window();
        self.rows
            .iter()
            .rev()
            .find(|r| r.resets_clock(window))
            .map(|r| r.created_at)
    }

    /// Até quando o passo em andamento pode ser feito, com `seconds` por
    /// passo e `team_seconds` para a janela simultânea; `None` sem passo
    /// esperando.
    pub fn deadline(&self, seconds: i32, team_seconds: i32) -> Option<DateTime<Utc>> {
        let turn = self.state.turn();
        if !matches!(turn, Turn::Act { .. }) {
            return None;
        }
        let seconds = if turn.is_window() {
            team_seconds
        } else {
            seconds
        };
        Some(self.clock()? + chrono::Duration::seconds(i64::from(seconds)))
    }
}
