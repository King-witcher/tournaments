//! Veto de mapas e draft de campeões ao vivo, entre a série ficar pronta e o
//! envio de cada partida. As regras (sequência, vez, opções, regras de
//! repetição, sorteios) são do `engine` (`veto`); aqui ficam o histórico
//! gravado (`db`, `state`), o "Pronto" dos lados, o pedido de desfazer e o
//! relógio.
//!
//! - A série pronta, num torneio com alguma fase de veto, pede que os dois
//!   lados cliquem em "Pronto" (`handlers::ready`); o veto começa quando os
//!   dois confirmaram, ou quando o organizador força (`handlers::start`).
//! - Tudo o que é do lado (Pronto, veto de mapas, bans, voto, pick do duelo,
//!   pedir e responder o desfazer) é de qualquer jogador do time: vale o
//!   primeiro clique, e o histórico grava quem fez. A janela simultânea do
//!   TDM é de cada jogador.
//! - Cada passo (`handlers::step`) passa pelo `engine` com as opções de agora
//!   (`Ctx`: os campeões do torneio e os já usados fora da série, pelo escopo
//!   da regra) e entra no histórico. O draft de campeões é da série inteira,
//!   com a partida 1 aberta; o pick grava a partida a que se destina.
//! - A partida seguinte começa quando a anterior é enviada (`advance`,
//!   chamado pelo `settle`), depois de completar pelo sorteio o draft que o
//!   organizador deixou pela metade. A série que passa a ter outros lados
//!   perde o veto.
//! - O tempo de cada passo vence ao ler e antes de cada ação (`due`), e um
//!   relógio no `realtime` dorme até o próximo prazo (`clock`): no fim, a
//!   escolha é sorteada pelo `engine`, gravada com a hora do prazo.
//! - O envio de uma partida por um jogador exige o veto dela concluído e o
//!   mapa que ele deu (`check_report`). Quando o veto deixou enviar conta no
//!   prazo de confirmação (`released_at`), e o veto começado de outra série
//!   fecha a contestação tardia (`begun_at`).

mod advance;
mod db;
mod due;
pub(super) mod handlers;
mod state;
mod types;
mod view;

use chrono::{DateTime, Utc};
use engine::{
    champions::NoRepeatScope,
    core::{ChampionId, Match, MatchState, PlayerId},
    veto::{self as ev, Turn},
};
use state::Kind;

pub use advance::advance;
pub use db::{Champion, champions, events, lineups};
pub use due::{any_due, apply_due, clock};
pub use state::{MatchVeto, VetoRow, ev_side};
pub use types::{VetoStatus, VetoView};
pub use view::{status, view};

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
    /// séries da fase (só há uma): das encerradas, o que foi jogado; das em
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
        .find(|r| !matches!(r.kind, Kind::Ready | Kind::UndoRequest | Kind::UndoDeclined))
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
        return Err(ApiError::invalid("match.map_mismatch"));
    }
    Ok(())
}
