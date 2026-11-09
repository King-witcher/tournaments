//! Veto de mapas e draft de campeões ao vivo, entre a série ficar pronta e o
//! envio de cada partida. As regras (sequência, vez, opções, regras de
//! repetição, sorteios) são do `engine` (`veto`); aqui ficam o histórico
//! gravado (`db`, `state`), o "Pronto" dos lados, o pedido de desfazer e o
//! relógio.
//!
//! - A série pronta, num torneio com alguma fase de veto, pede "Pronto"
//!   (`handlers::ready`): com tudo no capitão, dele; senão, de todos do elenco de
//!   agora (`Readiness`). Quem marcou desfaz enquanto o veto não começou
//!   (`handlers::unready`). O veto começa com os dois lados prontos, ou quando o
//!   organizador força (`handlers::start`).
//! - O que é do lado (veto de mapas, bans, voto, pick do duelo, pedir e responder
//!   o desfazer) é de quem a regra do torneio deixa (`acting`): vale o primeiro
//!   clique, e o histórico grava quem fez. O pick por jogador é de cada um ou do
//!   capitão, por todos (`PickOrder`: alternado nos times, ou a janela
//!   simultânea); `player` é de quem é o campeão, e `actor`, quem clicou.
//! - Às cegas, o que um lado escolheu fica escondido de quem não é dele até a
//!   janela fechar: quem monta o que sai é `view::build`, num ponto só.
//! - Cada passo (`handlers::step`) passa pelo `engine` com as opções de agora
//!   (`Ctx`: os campeões do torneio e os já usados fora da série, pelo escopo
//!   da regra) e entra no histórico. O draft de campeões é da série inteira,
//!   com a partida 1 aberta; o pick grava a partida a que se destina.
//! - A partida seguinte começa quando a anterior é enviada (`advance`,
//!   chamado pelo `settle`), depois de completar pelo sorteio o draft que o
//!   organizador deixou pela metade. A série que passa a ter outros lados
//!   perde o veto.
//! - O tempo de cada passo vence ao ler e antes de cada ação (`due`), e um
//!   relógio aplica nos torneios que ninguém leu (`clock`, chamado pelo
//!   `realtime`, que dorme até o próximo prazo; relê só os torneios que podem ter
//!   mudado): a ação vale até 2 s depois do prazo que a série mostra
//!   (`due::GRACE`); então a escolha é sorteada pelo `engine`, gravada com essa
//!   hora.
//! - A série diz qual vez mostra (`VetoView::step`, o evento que a abriu);
//!   a ação que manda esse número é recusada se a vez mudou antes de ela
//!   chegar (`veto.step_changed`), em vez de cair no passo seguinte.
//! - O envio de uma partida por um jogador exige o veto dela concluído e o
//!   mapa que ele deu (`check_report`). Quando o veto deixou enviar conta no
//!   prazo de confirmação (`released_at`), e o veto começado de outra série
//!   fecha a contestação tardia (`begun_at`).
//!
//! A sala de veto (`veto_rooms`) joga o mesmo veto sem torneio: usa o histórico
//! refeito (`VetoLog`), o passo conferido (`step`), os sorteios do fim do tempo
//! (`due_draws`) e a montagem da visão (`view::build`), e grava na tabela dela.

mod advance;
mod clock;
mod db;
mod due;
pub(super) mod handlers;
mod state;
pub mod step;
mod types;
mod view;

use chrono::{DateTime, Utc};
use engine::{
    champions::NoRepeatScope,
    core::{ChampionId, Match, MatchState, PlayerId},
    veto::{self as ev, Turn},
};

pub use advance::advance;
pub use clock::{VetoClock, VetoPass};
pub use db::{Champion, NewVetoEvent};
pub use due::{GRACE, any_due, apply_due, due_draws};
pub use state::{Kind, MatchVeto, MatchVetoRow, Readiness, VetoLog, VetoRow, ev_side, turn_action};
pub use types::{VetoChoice, VetoReadyBy, VetoStatus, VetoView};
pub use view::{Board, Permissions, Shown, build, next_map, status, view};

use super::stage::Stage;
use crate::shared::ApiError;

/// O que as opções de campeão precisam de fora da série.
pub struct Ctx {
    champions: Vec<ChampionId>,
    used: [Vec<ChampionId>; 2],
    used_by: Vec<(PlayerId, ChampionId)>,
}

impl Ctx {
    /// Para a série do veto `mv` (ver `between`).
    pub fn of(stage: &Stage, mv: &MatchVeto) -> Self {
        Self::between(stage, mv.match_id, mv.sides)
    }

    /// Para a série `id`, entre `sides`: os campeões ativos do torneio e, com a
    /// regra de não repetição na fase ou no torneio, o que conta das outras
    /// séries da fase da série (as fases são isoladas: `tournament` vale como
    /// `stage`, e nada vem das fases anteriores): das encerradas, o que foi jogado; das em
    /// andamento, também o que já está escolhido para as partidas por jogar,
    /// reservado (duas séries ao mesmo tempo não usam o mesmo campeão, e a
    /// reserva sai quando a série acaba sem a partida). No TDM, pela regra do
    /// time, o que o time escolheu; pela do jogador, o que cada jogador
    /// escolheu (por quem ele jogava, inclusive outro time).
    pub fn between(
        stage: &Stage,
        id: engine::core::MatchId,
        sides: (engine::core::ParticipantId, engine::core::ParticipantId),
    ) -> Self {
        let mut used: [Vec<ChampionId>; 2] = [Vec::new(), Vec::new()];
        let mut used_by: Vec<(PlayerId, ChampionId)> = Vec::new();
        if let Some(rule) = stage.veto_rules.champions.no_repeat
            && rule.scope != NoRepeatScope::Series
        {
            for other in stage.vetoes.iter().filter(|v| v.match_id != id) {
                let teams = [other.sides.0, other.sides.1];
                for game in 1..=other.state.best_of() {
                    for a in other.state.counted_picks(game) {
                        let Some(c) = a.champion() else {
                            continue;
                        };
                        let team = teams[usize::from(a.side == engine::series::Side::B)];
                        if team == sides.0 {
                            used[0].push(c);
                        } else if team == sides.1 {
                            used[1].push(c);
                        }
                        if let Some(player) = a.player {
                            used_by.push((player, c));
                        }
                    }
                }
            }
        }
        Self {
            champions: stage
                .champions
                .iter()
                .filter(|c| c.active)
                .map(|c| c.id)
                .collect(),
            used,
            used_by,
        }
    }

    /// Fora de um torneio (a sala de veto): os campeões ativos, e nada usado
    /// fora da série.
    pub fn alone(champions: &[Champion]) -> Self {
        Self {
            champions: champions
                .iter()
                .filter(|c| c.active)
                .map(|c| c.id)
                .collect(),
            used: [Vec::new(), Vec::new()],
            used_by: Vec::new(),
        }
    }

    pub fn engine(&self) -> ev::Context<'_> {
        ev::Context {
            champions: &self.champions,
            used: [&self.used[0], &self.used[1]],
            used_by: &self.used_by,
        }
    }
}

/// Para o prazo de confirmação (`release`): quando o veto da partida `id`
/// mudou pela última vez (começo, jogo seguinte, passo, desfazer; o
/// "Pronto" e o pedido de desfazer não contam). Com o envio liberado pelo
/// `check_report`, é quando o veto deixou enviar. `None` sem veto.
pub fn released_at(stage: &Stage, id: engine::core::MatchId) -> Option<DateTime<Utc>> {
    stage
        .veto_of(id)?
        .rows
        .iter()
        .rev()
        .find(|r| !r.kind.api_only())
        .map(|r| r.created_at)
}

/// Quando o veto da partida `id` começou; `None` se não começou.
pub fn begun_at(stage: &Stage, id: engine::core::MatchId) -> Option<DateTime<Utc>> {
    stage
        .veto_of(id)?
        .rows
        .iter()
        .find(|r| r.kind == Kind::Begin)
        .map(|r| r.created_at)
}

const MISMATCH: ApiError = ApiError::invalid("match.map_mismatch");

/// O jogador pode enviar o mapa `game` da partida `m` com o mapa `map`? Sem
/// fase de veto no torneio, sempre. Com ela, o veto daquele jogo precisa ter
/// acabado (409 `match.veto_pending`) e o mapa ser o que ele deu (422
/// `match.map_mismatch`; no modo desligado, qualquer um). Jogo que já
/// passou no veto (anulado para ser jogado de novo) vale.
pub fn check_report(
    stage: &Stage,
    m: &Match,
    game: i16,
    map: Option<engine::core::MapId>,
) -> Result<(), ApiError> {
    if !stage.veto_rules.has_phases() || !matches!(m.state, MatchState::Ready { .. }) {
        return Ok(());
    }
    const PENDING: ApiError = ApiError::conflict("match.veto_pending");
    let mv = stage.veto_of(m.id).ok_or(PENDING)?;
    let current = i16::from(mv.state.game().ok_or(PENDING)?);
    let done = match mv.state.turn() {
        Turn::Done { .. } => true,
        Turn::Act { .. } | Turn::NotBegun | Turn::RostersUnknown { .. } => false,
    };
    if game > current || (game == current && !done) {
        return Err(PENDING);
    }
    let vetoed = u8::try_from(game).ok().and_then(|g| mv.state.map_of(g));
    if let Some(vetoed) = vetoed
        && Some(vetoed) != map
    {
        return Err(MISMATCH);
    }
    Ok(())
}

/// O resultado de uma partida já jogada fica no mapa que o veto deu a ela, se
/// deu, também com a série decidida (a contestação tardia): 422 `match.map_mismatch`.
pub fn check_map(
    stage: &Stage,
    id: engine::core::MatchId,
    game: i16,
    map: engine::core::MapId,
) -> Result<(), ApiError> {
    let vetoed = stage
        .veto_of(id)
        .zip(u8::try_from(game).ok())
        .and_then(|(mv, g)| mv.state.map_of(g));
    match vetoed {
        Some(vetoed) if vetoed != map => Err(MISMATCH),
        _ => Ok(()),
    }
}
