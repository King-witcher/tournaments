//! Os picks por jogador (`VetoRules::per_player`): depois dos bans da série
//! (alternados entre os lados), cada jogador do elenco escolhe o próprio
//! campeão de cada partida da série, na ordem de `PickOrder`:
//!
//! - **Alternado**, nos times: partida a partida, um pick por vez, os lados
//!   em O, X, X, O, O, X, X, O... (`alternating_turn`), com quem abre (O)
//!   trocando a cada partida, como no duelo. Na vez do lado, qualquer um dele
//!   que ainda não escolheu naquela partida escolhe o próprio campeão; cada
//!   pick é um passo, com o prazo do passo, e no fim do prazo é sorteado para
//!   o primeiro do lado sem escolha, na ordem do elenco (`due_picks`). Um lado
//!   desfalcado perde as posições que não tem quem ocupe.
//! - **Janela** (`Blind`, `Open`): sem vez e em qualquer ordem, num prazo só.
//!   Ela fecha quando todo o elenco escolheu para todas as partidas; no fim do
//!   prazo, cada escolha que falta é sorteada (`random_team_choice`). Às
//!   cegas, em qualquer tamanho e só com espelho, os picks de um lado ficam
//!   escondidos dos outros até ela fechar (`sealed_games`, que a API aplica ao
//!   montar o que cada um vê) e não se desfazem (`can_undo`): quem desfez teria
//!   de reabrir uma janela já vista. Aberta (`Open`), só nas séries que
//!   começaram antes de a ordem existir, todos veem tudo.
//!
//! - **Elenco** (`Veto::set_rosters`, `Rosters`): quem escolhe de cada lado
//!   agora, que a API passa (os jogadores do time que estão na comunidade),
//!   e a escalação de cada partida enviada. Sem ele, a vez é
//!   `Turn::RostersUnknown`, nunca uma partida acabada. A escolha de quem
//!   saiu do elenco continua no histórico, mas não conta (ver "Pick que
//!   conta" em `draft.rs`); quem entrou passa a faltar nas partidas ainda
//!   abertas (da em andamento até a última), e os picks reabrem para ele na
//!   partida em andamento: partida enviada, ou a série encerrada, não reabre.
//! - **Opções de um jogador** (`team_options`): as de `pick_options` (ver
//!   `draft.rs`). Cada jogador cede sozinho, na hora em que escolhe: com o
//!   pool no limite, quem escolhe primeiro pode pegar o último campeão livre
//!   e o seguinte ficar com a regra cedida (o pick grava que cedeu). Sem
//!   espelho, fica com o campeão quem confirma primeiro, também entre times.

use std::collections::HashSet;

use super::{
    Action, Applied, Choice, Context, Phase, PickOptions, PickOrder, Picker, Side, Target, Turn,
    Veto, VetoError, draw, index,
    sequence::{game_opener, pick_side},
};
use crate::core::{ChampionId, PlayerId};

/// O que só a API sabe agora: o histórico do veto não diz quem está no time
/// nem o que já foi enviado.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rosters {
    /// Quem escolhe campeão de cada lado (`[a, b]`) no pick por jogador: os
    /// jogadores do time que estão na comunidade, na ordem do sorteio do
    /// tempo esgotado. Cada um uma vez só, no máximo `team_size` por lado.
    pub players: [Vec<PlayerId>; 2],
    /// As partidas já enviadas, na ordem: quem jogou cada uma por lado (a
    /// escalação gravada no envio). A janela não reabre nelas, e o pick de
    /// quem não a jogou não conta.
    pub lineups: Vec<[Vec<PlayerId>; 2]>,
    /// A série acabou (decidida ou W.O.): nenhuma escolha reabre.
    pub finished: bool,
}

/// Quem falta escolher, e para qual partida.
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

    /// A partida em andamento, se ela chegou aos picks por jogador, sem olhar
    /// o elenco: o `replay` aplica as escolhas sem ele.
    fn team_phase(&self) -> Option<u8> {
        match self.phase() {
            Phase::TeamPick(game) => Some(game),
            Phase::Turn(_) => None,
        }
    }

    /// A vez da partida `game`, nos picks por jogador (ver o `//!`).
    pub(super) fn team_turn(&self, game: u8) -> Turn {
        if self.rosters.is_none() {
            return Turn::RostersUnknown { game };
        }
        let missing = self.missing_now();
        if missing.is_empty() {
            return Turn::Done { game };
        }
        let (sides, pick_for) = match self.alternating_turn(&missing) {
            Some((side, pick_for)) => (vec![side], Some(pick_for)),
            None => {
                let mut sides: Vec<Side> = missing.iter().map(|m| m.side).collect();
                sides.dedup();
                (sides, None)
            }
        };
        Turn::Act {
            game,
            action: Action::TeamPick,
            target: Target::Champion,
            sides,
            pick_for,
        }
    }

    /// Na ordem alternada, de quem é a vez com `missing` por escolher, e para
    /// qual partida: a primeira em que falta alguém e, nela, o lado que o
    /// padrão (`pick_side`) pede depois do que já foi escolhido. A posição de
    /// um lado sem ninguém por escolher fica para o outro. `None` na janela.
    fn alternating_turn(&self, missing: &[Missing]) -> Option<(Side, u8)> {
        if self.rules.pick_order != PickOrder::Alternating {
            return None;
        }
        let first = self.begin?.first;
        let game = missing.iter().map(|m| m.game).min()?;
        let opener = game_opener(first, game);
        let mut done = [0usize; 2];
        for a in self.counted_picks(game) {
            done[index(a.side)] += 1;
        }
        let positions = usize::from(self.rules.team_size) * 2;
        for side in (0..positions).map(|i| pick_side(opener, i)) {
            if done[index(side)] > 0 {
                done[index(side)] -= 1;
            } else if missing.iter().any(|m| m.side == side && m.game == game) {
                return Some((side, game));
            }
        }
        // O elenco não passa do tamanho do time: o padrão cobre todos.
        missing
            .iter()
            .find(|m| m.game == game)
            .map(|m| (m.side, game))
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

    /// Os picks por jogador para a partida `game` que contam (ver "Pick que
    /// conta" em `draft.rs`), na ordem.
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

    /// Quem ainda falta nos picks por jogador, e para qual partida; vazio fora
    /// deles (outra fase, duelo alternado) e sem o elenco informado.
    pub fn missing(&self) -> Vec<Missing> {
        if self.team_phase().is_some() && self.rosters.is_some() {
            self.missing_now()
        } else {
            Vec::new()
        }
    }

    /// Quem pode escolher agora, e para qual partida: na janela, todos os que
    /// faltam; na ordem alternada, os do lado da vez sem escolha na partida da
    /// vez, na ordem do elenco.
    pub fn pickable(&self) -> Vec<Missing> {
        let missing = self.missing();
        match self.alternating_turn(&missing) {
            Some((side, game)) => missing
                .into_iter()
                .filter(|m| m.side == side && m.game == game)
                .collect(),
            None => missing,
        }
    }

    /// O que o fim do prazo sorteia agora, na ordem em que aplicar: na janela,
    /// tudo o que falta; na ordem alternada, o pick do primeiro do lado da vez.
    pub fn due_picks(&self) -> Vec<Missing> {
        let mut picks = self.pickable();
        if self.rules.pick_order == PickOrder::Alternating {
            picks.truncate(1);
        }
        picks
    }

    /// As partidas cujos picks ficam escondidos de quem não é do lado (só à
    /// vista dele, ver o `//!`): com a janela às cegas aberta, as que ainda
    /// aceitam escolha. Vazio nas outras ordens e com a janela fechada. Sem o
    /// elenco informado, que a API sempre informa, a janela conta como aberta.
    pub fn sealed_games(&self) -> Vec<u8> {
        if self.rules.pick_order != PickOrder::Blind || !self.begun() {
            return Vec::new();
        }
        let open = match self.turn() {
            Turn::RostersUnknown { .. } => true,
            turn => turn.is_window(),
        };
        if !open {
            return Vec::new();
        }
        (1..=self.best_of).filter(|g| !self.closed(*g)).collect()
    }

    /// A escolha de `player` para a partida `game` (ver o `//!`). Sem `ctx`
    /// (no `replay`), só a fase, a partida e o ban são conferidos, e vale o
    /// `gave_way` gravado; com ele, antes, o elenco (`RostersUnknown`), a
    /// partida fechada (`Finished`), se o jogador é do elenco do lado
    /// (`NotYourTurn`), se ele já tem pick que conta para a partida
    /// (`AlreadyPicked`) e, na ordem alternada, se é a vez do lado nessa
    /// partida (`NotYourTurn`), e depois as opções, que dizem o que cedeu.
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
                if self
                    .alternating_turn(&self.missing_now())
                    .is_some_and(|turn| turn != (side, game))
                {
                    return Err(VetoError::NotYourTurn);
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
    /// `game` (vazio se não falta ele nela ou, na ordem alternada, se não é a
    /// vez dele) e o que cedeu.
    pub fn team_options(
        &self,
        side: Side,
        player: PlayerId,
        game: u8,
        ctx: &Context,
    ) -> PickOptions {
        let wanted = Missing { side, player, game };
        if !self.pickable().contains(&wanted) {
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

    /// O campeão de `player` para a partida `game` quando o prazo acaba: uma
    /// das `team_options`, sorteada pela seed da série e pela posição no
    /// histórico (quem chama aplica um de cada vez, então cada sorteio tem a
    /// sua). `None` se ele não pode escolher agora.
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
