//! A janela simultânea de campeões nos times (TDM): depois dos bans da série
//! (alternados entre os lados, como no duelo), cada jogador do elenco de
//! cada lado escolhe o próprio campeão de cada partida da série, sem vez e
//! em qualquer ordem, vendo as escolhas dos outros. A janela fecha quando
//! todo o elenco escolheu para todas as partidas; quem a API vê sem
//! escolher no fim do prazo recebe um campeão sorteado para cada partida que
//! falta (`random_team_choice`).
//!
//! - **Elenco** (`Veto::set_rosters`, `Rosters`): quem escolhe de cada lado
//!   agora, que a API passa (os jogadores do time que estão na comunidade),
//!   e a escalação de cada partida enviada. Sem ele, a vez é
//!   `Turn::RostersUnknown`, nunca uma partida acabada. A escolha de quem
//!   saiu do elenco continua no histórico, mas não conta (ver "Pick que
//!   conta" em `draft.rs`); quem entrou passa a faltar nas partidas ainda
//!   abertas (da em andamento até a última), e a janela reabre para ele na
//!   partida em andamento: partida enviada, ou a série encerrada, não reabre.
//! - **Opções de um jogador** (`team_options`): as de `pick_options` (ver
//!   `draft.rs`). Cada jogador cede sozinho, na hora em que escolhe: com o
//!   pool no limite, quem escolhe primeiro pode pegar o último campeão livre
//!   e o seguinte ficar com a regra cedida (o pick grava que cedeu). Sem
//!   espelho, fica com o campeão quem confirma primeiro, também entre times.

use std::collections::HashSet;

use super::{
    Action, Applied, Choice, Context, Phase, Side, Target, Turn, Veto, VetoError, draw, index,
};
use super::{PickOptions, Picker};
use crate::core::{ChampionId, PlayerId};

/// O que só a API sabe agora: o histórico do veto não diz quem está no time
/// nem o que já foi enviado.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rosters {
    /// Quem escolhe campeão de cada lado (`[a, b]`) nos times: os jogadores
    /// do time que estão na comunidade, na ordem do sorteio do tempo
    /// esgotado. Cada um uma vez só, no máximo `team_size` por lado.
    pub players: [Vec<PlayerId>; 2],
    /// As partidas já enviadas, na ordem: quem jogou cada uma por lado (a
    /// escalação gravada no envio). A janela não reabre nelas, e o pick de
    /// quem não a jogou não conta.
    pub lineups: Vec<[Vec<PlayerId>; 2]>,
    /// A série acabou (decidida ou W.O.): nenhuma escolha reabre.
    pub finished: bool,
}

/// Quem falta escolher na janela simultânea, e para qual partida.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Missing {
    pub side: Side,
    pub player: PlayerId,
    pub game: u8,
}

impl Veto {
    /// Informa o que o histórico não diz (ver `Rosters`). O `replay` não
    /// depende dele. Erros: `RosterRepeated` (o mesmo jogador duas vezes,
    /// num lado ou nos dois) e `RosterTooLarge` (mais que `team_size` num
    /// lado).
    pub fn set_rosters(&mut self, rosters: Rosters) -> Result<(), VetoError> {
        let mut seen = HashSet::new();
        for side in &rosters.players {
            if side.len() > usize::from(self.rules.team_size) {
                return Err(VetoError::RosterTooLarge {
                    max: self.rules.team_size,
                    got: side.len(),
                });
            }
            for p in side {
                if !seen.insert(*p) {
                    return Err(VetoError::RosterRepeated(*p));
                }
            }
        }
        self.rosters = Some(rosters);
        Ok(())
    }

    /// A partida em andamento, se ela chegou à janela simultânea, sem olhar
    /// o elenco: o `replay` aplica as escolhas sem ele.
    fn team_phase(&self) -> Option<u8> {
        match self.phase() {
            Phase::TeamPick(game) => Some(game),
            Phase::Turn(_) => None,
        }
    }

    /// A vez da partida `game`, na janela simultânea (ver o `//!`).
    pub(super) fn team_turn(&self, game: u8) -> Turn {
        if self.rosters.is_none() {
            return Turn::RostersUnknown { game };
        }
        let mut sides: Vec<Side> = self.missing_now().iter().map(|m| m.side).collect();
        sides.dedup();
        if sides.is_empty() {
            Turn::Done { game }
        } else {
            Turn::Act {
                game,
                action: Action::TeamPick,
                target: Target::Champion,
                sides,
                pick_for: None,
            }
        }
    }

    /// A partida `game` não aceita mais escolha: veio outra depois dela, foi
    /// enviada ou a série acabou (pelo que a API informou).
    pub(super) fn closed(&self, game: u8) -> bool {
        self.game().is_some_and(|current| game < current)
            || self
                .rosters
                .as_ref()
                .is_some_and(|r| r.finished || usize::from(game) <= r.lineups.len())
    }

    /// Os jogadores do lado `side` no elenco informado (vazio sem ele).
    pub(super) fn roster(&self, side: Side) -> &[PlayerId] {
        self.rosters
            .as_ref()
            .map_or(&[], |r| r.players[index(side)].as_slice())
    }

    /// Quem jogou a partida enviada `game`, por lado; `None` se ela não foi
    /// enviada (ou sem o elenco informado).
    pub(super) fn lineup(&self, game: u8) -> Option<&[Vec<PlayerId>; 2]> {
        let i = usize::from(game).checked_sub(1)?;
        self.rosters.as_ref()?.lineups.get(i)
    }

    /// As escolhas da janela simultânea para a partida `game` que contam
    /// (ver "Pick que conta" em `draft.rs`), na ordem.
    pub fn team_picks(&self, game: u8) -> Vec<Applied> {
        self.counted_picks(game)
            .into_iter()
            .filter(|s| s.action == Action::TeamPick)
            .collect()
    }

    /// Quem do elenco ainda não tem pick que conta para alguma partida
    /// aberta, sem olhar a fase: o lado A primeiro, na ordem do elenco e das
    /// partidas.
    pub(super) fn missing_now(&self) -> Vec<Missing> {
        let current = self.game().unwrap_or(1);
        let open: Vec<u8> = (current..=self.best_of)
            .filter(|g| !self.closed(*g))
            .collect();
        let counted = self.counted();
        let mut missing = Vec::new();
        for side in [Side::A, Side::B] {
            for player in self.roster(side) {
                for game in &open {
                    let picked = counted
                        .iter()
                        .any(|a| a.player == Some(*player) && a.game == Some(*game));
                    if !picked {
                        missing.push(Missing {
                            side,
                            player: *player,
                            game: *game,
                        });
                    }
                }
            }
        }
        missing
    }

    /// Quem ainda falta na janela simultânea, e para qual partida; vazio fora
    /// dela (outra fase, duelo) e sem o elenco informado.
    pub fn missing(&self) -> Vec<Missing> {
        if self.team_phase().is_some() && self.rosters.is_some() {
            self.missing_now()
        } else {
            Vec::new()
        }
    }

    /// A escolha de `player` para a partida `game` (ver o `//!`). Sem `ctx`
    /// (no `replay`), só a fase, a partida e o ban são conferidos, e vale o
    /// `gave_way` gravado; com ele, antes, o elenco (`RostersUnknown`), a
    /// partida fechada (`Finished`), se o jogador é do elenco do lado
    /// (`NotYourTurn`) e se ele já tem pick que conta para a partida
    /// (`AlreadyPicked`), e depois as opções, que dizem o que cedeu.
    pub(super) fn team_step(
        &mut self,
        side: Side,
        player: PlayerId,
        game: u8,
        champion: ChampionId,
        gave_way: bool,
        ctx: Option<&Context>,
    ) -> Result<(), VetoError> {
        let Some(current) = self.team_phase() else {
            return Err(match self.turn() {
                Turn::NotBegun => VetoError::NotBegun,
                Turn::Done { .. } => VetoError::Finished,
                Turn::Act { .. } | Turn::RostersUnknown { .. } => VetoError::WrongAction,
            });
        };
        if game == 0 || game > self.best_of {
            return Err(VetoError::OptionInvalid);
        }
        if game < current {
            return Err(VetoError::Finished);
        }
        let picker = Picker {
            side,
            player: Some(player),
        };
        let gave_way = match ctx {
            Some(ctx) => {
                if self.rosters.is_none() {
                    return Err(VetoError::RostersUnknown);
                }
                if self.closed(game) {
                    return Err(VetoError::Finished);
                }
                if !self.roster(side).contains(&player) {
                    return Err(VetoError::NotYourTurn);
                }
                if self
                    .counted_picks(game)
                    .iter()
                    .any(|a| a.player == Some(player))
                {
                    return Err(VetoError::AlreadyPicked);
                }
                let options = self.pick_options(picker, game, ctx);
                if !options.champions.contains(&champion) {
                    return Err(self.rejection(picker, game, champion, ctx));
                }
                options.gave_way.is_some()
            }
            None => {
                if self.bans().iter().any(|b| b.champion() == Some(champion)) {
                    return Err(VetoError::OptionInvalid);
                }
                gave_way
            }
        };
        let current = self.games.last_mut().ok_or(VetoError::NotBegun)?;
        current.steps.push(Applied {
            side,
            action: Action::TeamPick,
            choice: Choice::Champion(champion),
            player: Some(player),
            game: Some(game),
            gave_way,
        });
        Ok(())
    }

    /// O que `player`, do lado `side`, pode escolher agora para a partida
    /// `game` (vazio se não falta ele nela) e o que cedeu.
    pub fn team_options(
        &self,
        side: Side,
        player: PlayerId,
        game: u8,
        ctx: &Context,
    ) -> PickOptions {
        let wanted = Missing { side, player, game };
        if !self.missing().contains(&wanted) {
            return PickOptions::default();
        }
        self.pick_options(
            Picker {
                side,
                player: Some(player),
            },
            game,
            ctx,
        )
    }

    /// O campeão de `player` para a partida `game` quando o prazo da janela
    /// acaba: uma das `team_options`, sorteada pela seed da série e pela
    /// posição no histórico (quem chama aplica um de cada vez, então cada
    /// sorteio tem a sua). `None` se não falta ele nela.
    pub fn random_team_choice(
        &self,
        side: Side,
        player: PlayerId,
        game: u8,
        ctx: &Context,
    ) -> Option<ChampionId> {
        let options = self.team_options(side, player, game, ctx).champions;
        if options.is_empty() {
            return None;
        }
        let seed = self.begin?.seed;
        let i = draw::index(
            seed,
            draw::Purpose::TeamTimeout,
            game,
            self.events,
            options.len(),
        );
        options.get(i).copied()
    }
}
