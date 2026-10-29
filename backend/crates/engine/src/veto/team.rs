//! A escolha simultânea de campeões nos times (TDM): depois dos bans (os
//! mesmos do duelo, alternados entre os lados), cada jogador do elenco de
//! cada lado escolhe o próprio campeão, sem vez, vendo as escolhas dos
//! outros. O jogo acaba quando todo o elenco escolheu; quem a API vê sem
//! escolher no fim do prazo recebe um campeão sorteado (`random_team_choice`).
//!
//! - **Elenco** (`Veto::set_rosters`, `Rosters`): quem escolhe de cada lado
//!   no jogo em andamento, que a API passa (os jogadores do time que estão na
//!   comunidade agora), e quantos jogos da série já foram enviados. Sem ele,
//!   a vez é `Turn::RostersUnknown`, nunca um jogo acabado. A escolha de
//!   quem saiu do elenco continua no histórico; quem entrou passa a faltar,
//!   mas só no jogo em andamento ainda não enviado: jogo enviado, ou a série
//!   encerrada, não reabre.
//! - **Opções de um jogador** (`team_options`): os campeões sem ban no mapa,
//!   menos os que um colega do elenco de agora já escolheu nele (sem
//!   `team_duplicates`; a escolha de quem saiu não conta, porque não chegou a
//!   ser jogada), menos os que a regra de não repetição tira dele
//!   (`forbidden_for`). Se a regra não deixa nenhum, ela cede; se nem sem
//!   colegas sobra, repete no time. As opções dizem o que cedeu
//!   (`TeamOptions::gave_way`), para a tela avisar. Cada jogador cede
//!   sozinho, na hora em que escolhe: com o pool no limite, quem escolhe
//!   primeiro pode pegar o último campeão livre e o seguinte ficar com a
//!   regra cedida.
//! - **Regra de não repetição**: a do jogador tira o que ele usou nos mapas
//!   anteriores da série e, fora dela, o que a API passa (`Context::used_by`);
//!   a do time tira o que qualquer um do time usou nos mapas anteriores,
//!   inclusive quem saiu do time depois, e o de fora (`Context::used`).
//! - **Proteção do ban** (`picks_needed`, `pickable_sets`): o lado que bane
//!   precisa que sobrem campeões para o elenco todo (um só, se pode repetir
//!   no time), contados para cada jogador (pela regra do jogador, cada um
//!   tem os seus).

use std::collections::HashSet;

use super::{
    Action, Applied, Choice, Context, Phase, Side, Target, Turn, Veto, VetoError, draw, index,
};
use crate::{
    champions::NoRepeatWho,
    core::{ChampionId, PlayerId},
};

/// O que só a API sabe agora, nos times: o histórico do veto não diz quem
/// está no time nem se o jogo em andamento já foi enviado.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rosters {
    /// Quem escolhe campeão de cada lado (`[a, b]`): os jogadores do time que
    /// estão na comunidade, na ordem do sorteio do tempo esgotado. Cada um
    /// uma vez só, no máximo `team_size` por lado.
    pub players: [Vec<PlayerId>; 2],
    /// Jogos da série já enviados: a escolha simultânea de um deles não
    /// reabre com quem entrou no time depois.
    pub sent: u8,
    /// A série acabou (decidida ou W.O.): nenhuma escolha reabre.
    pub finished: bool,
}

/// O que cedeu para um jogador ter campeão na escolha simultânea.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GaveWay {
    /// A regra de não repetição: com ela, não sobrava nenhum; as opções
    /// trazem os que ela tiraria.
    Rule,
    /// Também o "sem repetir no time": nem sem a regra sobrava um que os
    /// colegas não escolheram no mapa; as opções são todos os sem ban.
    Team,
}

/// As opções de um jogador na escolha simultânea (`Veto::team_options`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TeamOptions {
    /// Na ordem do torneio; vazio se não falta ele.
    pub champions: Vec<ChampionId>,
    /// `None` se nada cedeu.
    pub gave_way: Option<GaveWay>,
}

impl Veto {
    /// O jogo em andamento, se ele chegou à escolha simultânea (nos times
    /// com a fase de campeões, o veto de mapa dele e os bans acabaram), sem
    /// olhar o elenco: o `replay` aplica as escolhas sem ele.
    fn team_phase(&self) -> Option<u8> {
        match self.phase() {
            Phase::TeamPick(game) => Some(game),
            Phase::Turn(_) => None,
        }
    }

    /// A vez do jogo `game`, na escolha simultânea (ver o `//!`).
    pub(super) fn team_turn(&self, game: u8) -> Turn {
        if self.rosters.is_none() {
            return Turn::RostersUnknown { game };
        }
        let sides = self.missing_sides(game);
        if self.closed(game) || sides.is_empty() {
            Turn::Done { game }
        } else {
            Turn::Act {
                game,
                action: Action::TeamPick,
                target: Target::Champion,
                sides,
            }
        }
    }

    /// O jogo `game` não aceita mais escolha: veio outro depois dele, foi
    /// enviado ou a série acabou (pelo que a API informou).
    fn closed(&self, game: u8) -> bool {
        self.game().is_some_and(|current| game < current)
            || self
                .rosters
                .as_ref()
                .is_some_and(|r| r.finished || game <= r.sent)
    }

    /// Os jogadores do lado `side` no elenco informado (vazio sem ele).
    fn roster(&self, side: Side) -> &[PlayerId] {
        self.rosters
            .as_ref()
            .map_or(&[], |r| r.players[index(side)].as_slice())
    }

    /// A escolha de `player` (ver o `//!`). Sem `ctx` (no `replay`), só a
    /// fase, o ban e a escolha repetida do mesmo jogador são conferidos; com
    /// ele, antes, o elenco (`RostersUnknown`), o jogo fechado (`Finished`) e
    /// se o jogador é do elenco do lado (`NotYourTurn`), e depois as opções.
    pub(super) fn team_step(
        &mut self,
        side: Side,
        player: PlayerId,
        champion: ChampionId,
        ctx: Option<&Context>,
    ) -> Result<(), VetoError> {
        let Some(game) = self.team_phase() else {
            return Err(match self.turn() {
                Turn::NotBegun => VetoError::NotBegun,
                Turn::Done { .. } => VetoError::Finished,
                Turn::Act { .. } | Turn::RostersUnknown { .. } => VetoError::WrongAction,
            });
        };
        if ctx.is_some() {
            if self.rosters.is_none() {
                return Err(VetoError::RostersUnknown);
            }
            if self.closed(game) {
                return Err(VetoError::Finished);
            }
            if !self.roster(side).contains(&player) {
                return Err(VetoError::NotYourTurn);
            }
        }
        if self
            .team_picks(game)
            .iter()
            .any(|a| a.player == Some(player))
        {
            return Err(VetoError::AlreadyPicked);
        }
        match ctx {
            Some(ctx) => {
                if !self
                    .team_options(side, player, ctx)
                    .champions
                    .contains(&champion)
                {
                    let forbidden = ctx.champions.contains(&champion)
                        && !self.banned(game).contains(&champion)
                        && !self.mates_picks(game, side, player).contains(&champion)
                        && self.forbidden_for(side, player, ctx).contains(&champion);
                    return Err(if forbidden {
                        VetoError::ChampionForbidden
                    } else {
                        VetoError::OptionInvalid
                    });
                }
            }
            None => {
                if self.banned(game).contains(&champion) {
                    return Err(VetoError::OptionInvalid);
                }
            }
        }
        let current = self.games.last_mut().ok_or(VetoError::NotBegun)?;
        current.steps.push(Applied {
            side,
            action: Action::TeamPick,
            choice: Choice::Champion(champion),
            player: Some(player),
        });
        Ok(())
    }

    /// As escolhas simultâneas do jogo `game`, na ordem (inclusive as de
    /// quem saiu do elenco depois).
    pub fn team_picks(&self, game: u8) -> Vec<Applied> {
        self.champion_steps(game)
            .iter()
            .filter(|s| s.action == Action::TeamPick)
            .copied()
            .collect()
    }

    /// Quem do elenco ainda não escolheu no jogo `game` (o em andamento),
    /// sem olhar a fase nem se o jogo fechou: o lado A primeiro, na ordem do
    /// elenco.
    fn missing_now(&self, game: u8) -> Vec<(Side, PlayerId)> {
        let picked: Vec<PlayerId> = self
            .team_picks(game)
            .iter()
            .filter_map(|a| a.player)
            .collect();
        [Side::A, Side::B]
            .into_iter()
            .flat_map(|side| {
                self.roster(side)
                    .iter()
                    .filter(|p| !picked.contains(p))
                    .map(move |p| (side, *p))
            })
            .collect()
    }

    /// Os lados em que falta alguém escolher no jogo `game` (para a vez).
    fn missing_sides(&self, game: u8) -> Vec<Side> {
        let mut sides: Vec<Side> = self.missing_now(game).into_iter().map(|(s, _)| s).collect();
        sides.dedup();
        sides
    }

    /// Quem ainda falta escolher na escolha simultânea do jogo `game`; vazio
    /// fora dela (outro jogo, outra fase, duelo), sem o elenco informado e
    /// com o jogo fechado (enviado, série encerrada).
    pub fn missing(&self, game: u8) -> Vec<(Side, PlayerId)> {
        if self.team_phase() == Some(game) && self.rosters.is_some() && !self.closed(game) {
            self.missing_now(game)
        } else {
            Vec::new()
        }
    }

    /// O que os colegas de `player` (do lado `side`) que estão no elenco
    /// escolheram no jogo `game`, quando o time não pode repetir no mapa. A
    /// escolha de quem saiu do elenco não conta: ela não chegou a ser jogada.
    fn mates_picks(&self, game: u8, side: Side, player: PlayerId) -> HashSet<ChampionId> {
        if self.rules.team_duplicates {
            return HashSet::new();
        }
        let roster = self.roster(side);
        self.team_picks(game)
            .iter()
            .filter(|a| a.side == side)
            .filter(|a| a.player.is_some_and(|p| p != player && roster.contains(&p)))
            .filter_map(|a| match a.choice {
                Choice::Champion(c) => Some(c),
                Choice::Map(_) => None,
            })
            .collect()
    }

    /// O que o lado `side` (pela regra do time) ou `player` (pela do
    /// jogador; `None`: qualquer um, inclusive quem saiu do time depois)
    /// escolheu nos jogos da série antes do em andamento.
    fn earlier_picks(&self, side: Side, player: Option<PlayerId>) -> HashSet<ChampionId> {
        let current = self.game().unwrap_or(0);
        (1..current)
            .flat_map(|g| self.team_picks(g))
            .filter(|a| a.side == side && (player.is_none() || a.player == player))
            .filter_map(|a| match a.choice {
                Choice::Champion(c) => Some(c),
                Choice::Map(_) => None,
            })
            .collect()
    }

    /// O que a regra de não repetição tira de `player`, do lado `side`, no
    /// jogo em andamento.
    pub fn forbidden_for(
        &self,
        side: Side,
        player: PlayerId,
        ctx: &Context,
    ) -> HashSet<ChampionId> {
        match ctx.no_repeat {
            None => HashSet::new(),
            Some(NoRepeatWho::Team) => self.side_used(side, ctx),
            Some(NoRepeatWho::Player) => {
                let mut used = self.earlier_picks(side, Some(player));
                used.extend(
                    ctx.used_by
                        .iter()
                        .filter(|(p, _)| *p == player)
                        .map(|(_, c)| *c),
                );
                used
            }
        }
    }

    /// Pela regra do time: o que o lado usou fora da série e nos jogos
    /// anteriores dela.
    fn side_used(&self, side: Side, ctx: &Context) -> HashSet<ChampionId> {
        let mut used = self.earlier_picks(side, None);
        used.extend(ctx.used[index(side)].iter().copied());
        used
    }

    /// O que a regra tira do lado `side` nos times: pela do time, o que ele
    /// usou; pela do jogador, o que ela tira de algum do elenco (para mostrar
    /// o lado; as opções e a proteção do ban contam cada jogador).
    pub(super) fn team_forbidden(&self, side: Side, ctx: &Context) -> HashSet<ChampionId> {
        match ctx.no_repeat {
            None => HashSet::new(),
            Some(NoRepeatWho::Team) => self.side_used(side, ctx),
            Some(NoRepeatWho::Player) => self
                .roster(side)
                .iter()
                .flat_map(|p| self.forbidden_for(side, *p, ctx))
                .collect(),
        }
    }

    /// Campeões que cada jogador do lado `side` precisa que sobrem no mapa
    /// (a proteção do ban): um no duelo ou podendo repetir no time; senão,
    /// um por jogador do elenco (sem o elenco informado, `team_size`).
    pub(super) fn picks_needed(&self, side: Side) -> usize {
        if !self.rules.teams() || self.rules.team_duplicates {
            return 1;
        }
        match &self.rosters {
            Some(r) => r.players[index(side)].len().max(1),
            None => usize::from(self.rules.team_size),
        }
    }

    /// Para a proteção do ban: o que cada jogador do lado `side` ainda pode
    /// escolher entre os `unbanned`, pela regra (no pior caso, todos ainda
    /// faltam escolher). No duelo, e nos times sem elenco, um conjunto só: o
    /// do lado.
    pub(super) fn pickable_sets(
        &self,
        side: Side,
        unbanned: &[ChampionId],
        ctx: &Context,
    ) -> Vec<HashSet<ChampionId>> {
        let without = |forbidden: HashSet<ChampionId>| -> HashSet<ChampionId> {
            unbanned
                .iter()
                .copied()
                .filter(|c| !forbidden.contains(c))
                .collect()
        };
        let roster = self.roster(side);
        if !self.rules.teams() || roster.is_empty() {
            return vec![without(self.forbidden(side, ctx))];
        }
        roster
            .iter()
            .map(|p| without(self.forbidden_for(side, *p, ctx)))
            .collect()
    }

    /// O que `player`, do lado `side`, pode escolher agora na escolha
    /// simultânea (vazio se não falta ele) e o que cedeu; ver o `//!`.
    pub fn team_options(&self, side: Side, player: PlayerId, ctx: &Context) -> TeamOptions {
        let Some(game) = self.team_phase() else {
            return TeamOptions::default();
        };
        if !self.missing(game).contains(&(side, player)) {
            return TeamOptions::default();
        }
        let banned = self.banned(game);
        let mates = self.mates_picks(game, side, player);
        let forbidden = self.forbidden_for(side, player, ctx);
        let unbanned: Vec<ChampionId> = ctx
            .champions
            .iter()
            .copied()
            .filter(|c| !banned.contains(c))
            .collect();
        let free: Vec<ChampionId> = unbanned
            .iter()
            .copied()
            .filter(|c| !mates.contains(c))
            .collect();
        let pickable: Vec<ChampionId> = free
            .iter()
            .copied()
            .filter(|c| !forbidden.contains(c))
            .collect();
        let (champions, gave_way) = if !pickable.is_empty() {
            (pickable, None)
        } else if !free.is_empty() {
            (free, Some(GaveWay::Rule))
        } else {
            (unbanned, Some(GaveWay::Team))
        };
        TeamOptions {
            gave_way: gave_way.filter(|_| !champions.is_empty()),
            champions,
        }
    }

    /// O campeão de `player` quando o prazo da escolha simultânea acaba: uma
    /// das `team_options`, sorteada pela seed da série e pela posição no
    /// histórico (quem chama aplica um de cada vez, então cada sorteio tem a
    /// sua). `None` se não falta ele.
    pub fn random_team_choice(
        &self,
        side: Side,
        player: PlayerId,
        ctx: &Context,
    ) -> Option<ChampionId> {
        let options = self.team_options(side, player, ctx).champions;
        if options.is_empty() {
            return None;
        }
        let seed = self.begin?.seed;
        let game = self.game()?;
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
