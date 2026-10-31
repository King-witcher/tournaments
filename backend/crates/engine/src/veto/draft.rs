//! O draft de campeões da série: os bans, os picks do duelo e o que cada um
//! pode escolher para uma partida (o mesmo vale na janela simultânea dos
//! times, `team.rs`).
//!
//! - **Opções de um pick** (`pick_options`): os campeões sem ban na série,
//!   menos os que outro já escolheu para a mesma partida (o adversário, sem
//!   `mirror`; um colega, sem `same_team`) e os que a regra de não repetição
//!   tira: o que quem escolhe (o jogador, pela regra do jogador nos times; o
//!   lado, no duelo e pela regra do time) escolheu para outra partida da
//!   série e o que a API passa de fora dela. Se a regra não deixa nenhum,
//!   ela cede (`GaveWay::Rule`); se nem sem ela sobra um, cede também a da
//!   partida (`GaveWay::Game`). As opções dizem o que cedeu, para a tela
//!   avisar.
//! - **Pick que conta** (`counted_picks`): o que é ou vai ser jogado. Numa
//!   partida enviada, o de quem a jogou (`Rosters::lineups`, a escalação do
//!   envio): quem saiu antes do envio não volta a contar. Numa partida
//!   aberta, o de quem está no elenco agora, e só se ainda vale: quem saiu e
//!   voltou perde o pick antigo se, enquanto esteve fora, outro escolheu sem
//!   ceder algo que não caberia com ele (um colega ou adversário na mesma
//!   partida, o time em outra pela regra do time); ele passa a faltar. Na
//!   série encerrada, a partida que não chegou a ser jogada não conta. No
//!   duelo, ninguém sai: contam todos, menos os dessa partida não jogada.
//! - **Proteção do ban** (`ban_options`): o lado que bane precisa que sobrem
//!   campeões para escolher todas as partidas (`picks_needed`, o mesmo pior
//!   caso de `champions::required_pool`), contados para cada um que escolhe
//!   (pela regra do jogador, cada um tem os seus). Se os bans que o
//!   adversário ainda tem bastariam para esgotar algum, o ban fica nos que
//!   nenhum desses pode escolher; se não há nenhum assim, a proteção cede,
//!   como a regra.

use std::collections::HashSet;

use super::{Action, Applied, Context, DraftStep, Veto, VetoError, draft_sequence, index, other};
use crate::{
    champions::NoRepeatWho,
    core::{ChampionId, PlayerId},
    series::Side,
};

/// Quem escolhe: o lado e, nos times, o jogador.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Picker {
    pub side: Side,
    pub player: Option<PlayerId>,
}

/// Por que um campeão não está nas opções de quem escolhe para uma partida,
/// na ordem em que se confere.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unavailable {
    /// Banido na série.
    Banned,
    /// Um colega já o escolheu para a partida (sem repetir no time).
    Teammate,
    /// Um adversário já o escolheu para a partida (sem espelho).
    Opponent,
    /// A regra de não repetição: escolhido para outra partida da série ou
    /// usado fora dela.
    Repeat,
}

/// O que cedeu para quem escolhe ter campeão.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GaveWay {
    /// A regra de não repetição: com ela, não sobrava nenhum; as opções
    /// trazem os que ela tiraria.
    Rule,
    /// Também a da partida (espelho, repetir no time): nem sem a regra
    /// sobrava um; as opções são todos os sem ban.
    Game,
}

/// As opções de um pick (`Veto::pick_options`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PickOptions {
    /// Na ordem do torneio.
    pub champions: Vec<ChampionId>,
    /// `None` se nada cedeu.
    pub gave_way: Option<GaveWay>,
}

/// O que tira cada campeão de um pick, sem ceder.
struct Excluded {
    banned: HashSet<ChampionId>,
    teammates: HashSet<ChampionId>,
    opponents: HashSet<ChampionId>,
    repeat: HashSet<ChampionId>,
}

impl Excluded {
    fn reason(&self, c: ChampionId) -> Option<Unavailable> {
        if self.banned.contains(&c) {
            Some(Unavailable::Banned)
        } else if self.teammates.contains(&c) {
            Some(Unavailable::Teammate)
        } else if self.opponents.contains(&c) {
            Some(Unavailable::Opponent)
        } else if self.repeat.contains(&c) {
            Some(Unavailable::Repeat)
        } else {
            None
        }
    }
}

impl Veto {
    /// Os passos do draft de campeões desta série (ver `draft_sequence`).
    pub(super) fn draft_sequence(&self) -> Vec<DraftStep> {
        let first = self.first().unwrap_or(Side::A);
        draft_sequence(self.rules.bans(), self.best_of, first, self.rules.teams())
    }

    /// Os passos de campeão da série, na ordem: o draft (feito com a partida
    /// 1 aberta) e, nos times, as escolhas de quem entrou no elenco depois.
    pub fn champion_steps(&self) -> Vec<Applied> {
        self.games
            .iter()
            .flat_map(|g| g.split().1.iter().copied())
            .collect()
    }

    /// Os bans da série, na ordem.
    pub fn bans(&self) -> Vec<Applied> {
        self.champion_steps()
            .into_iter()
            .filter(|s| s.action == Action::Ban)
            .collect()
    }

    fn banned(&self) -> HashSet<ChampionId> {
        self.bans().iter().filter_map(Applied::champion).collect()
    }

    /// Todos os picks da série, na ordem em que foram feitos.
    fn all_picks(&self) -> Vec<Applied> {
        self.champion_steps()
            .into_iter()
            .filter(|s| s.action != Action::Ban)
            .collect()
    }

    /// Os picks para a partida `game`, na ordem em que foram feitos (no
    /// duelo, um por lado; nos times, todos, também os que não contam).
    pub fn picks_for(&self, game: u8) -> Vec<Applied> {
        self.all_picks()
            .into_iter()
            .filter(|s| s.game == Some(game))
            .collect()
    }

    /// No duelo: o campeão de cada lado (`[a, b]`) na partida `game`.
    pub fn picks(&self, game: u8) -> [Option<ChampionId>; 2] {
        let mut picks = [None, None];
        for s in self.picks_for(game) {
            if s.player.is_none() {
                picks[index(s.side)] = s.champion();
            }
        }
        picks
    }

    /// A partida `game` não foi nem vai ser jogada: a série acabou antes.
    fn unplayed(&self, game: u8) -> bool {
        self.rosters
            .as_ref()
            .is_some_and(|r| r.finished && usize::from(game) > r.lineups.len())
    }

    /// Os picks que contam na série, na ordem em que foram feitos (ver o
    /// `//!`).
    pub(super) fn counted(&self) -> Vec<Applied> {
        let all = self.all_picks();
        if !self.rules.teams() {
            return all
                .into_iter()
                .filter(|a| !a.game.is_some_and(|g| self.unplayed(g)))
                .collect();
        }
        // Os de partidas enviadas, pela escalação, e os candidatos das
        // abertas, pelo elenco de agora; com a posição no histórico.
        let mut kept: Vec<(usize, Applied, bool)> = Vec::new();
        for (i, a) in all.iter().enumerate() {
            let (Some(game), Some(player)) = (a.game, a.player) else {
                continue;
            };
            if let Some(lineup) = self.lineup(game) {
                if lineup[index(a.side)].contains(&player) {
                    kept.push((i, *a, false));
                }
            } else if !self.unplayed(game)
                && (self.rosters.is_none() || self.roster(a.side).contains(&player))
            {
                kept.push((i, *a, true));
            }
        }
        kept.iter()
            .filter(|(i, a, open)| {
                !open || !kept.iter().any(|(j, b, _)| j > i && self.supersedes(b, a))
            })
            .map(|(_, a, _)| *a)
            .collect()
    }

    /// Nos times, `later` (feito depois) tira o lugar de `earlier`: é o pick
    /// do mesmo jogador para a mesma partida, ou, sem ter cedido nada, não
    /// caberia junto dele (então `earlier` era de quem estava fora do elenco).
    fn supersedes(&self, later: &Applied, earlier: &Applied) -> bool {
        if later.player == earlier.player {
            return later.game == earlier.game;
        }
        if later.gave_way || later.champion() != earlier.champion() {
            return false;
        }
        let rules = &self.rules.champions;
        if later.game == earlier.game {
            if later.side == earlier.side {
                !rules.same_team
            } else {
                !rules.mirror
            }
        } else {
            later.side == earlier.side && self.rules.no_repeat() == Some(NoRepeatWho::Team)
        }
    }

    /// Os picks para a partida `game` que contam (ver o `//!`): os que a API
    /// reserva fora da série enquanto ela não acabou.
    pub fn counted_picks(&self, game: u8) -> Vec<Applied> {
        self.counted()
            .into_iter()
            .filter(|a| a.game == Some(game))
            .collect()
    }

    /// Os picks da partida `game` se ela foi enviada, de quem a jogou (nos
    /// times, pela escalação); vazio se não foi.
    pub fn played_picks(&self, game: u8) -> Vec<Applied> {
        let sent = self
            .rosters
            .as_ref()
            .is_some_and(|r| usize::from(game) <= r.lineups.len());
        if !sent {
            return Vec::new();
        }
        let picks = self.picks_for(game);
        match self.lineup(game) {
            Some(lineup) if self.rules.teams() => picks
                .into_iter()
                .filter(|a| a.player.is_some_and(|p| lineup[index(a.side)].contains(&p)))
                .collect(),
            _ => picks,
        }
    }

    /// O que a regra de não repetição tira de `picker` na partida `game`, com
    /// os picks que contam (`counted`): o de fora da série e o que ele
    /// escolheu para as outras partidas dela (com `game` 0, todas).
    fn repeat_for(
        &self,
        picker: Picker,
        game: u8,
        counted: &[Applied],
        ctx: &Context,
    ) -> HashSet<ChampionId> {
        let Some(who) = self.rules.no_repeat() else {
            return HashSet::new();
        };
        let by_player = self.rules.teams() && who == NoRepeatWho::Player;
        let mut used: HashSet<ChampionId> = if by_player {
            ctx.used_by
                .iter()
                .filter(|(p, _)| Some(*p) == picker.player)
                .map(|(_, c)| *c)
                .collect()
        } else {
            ctx.used[index(picker.side)].iter().copied().collect()
        };
        for a in counted {
            if a.game != Some(game)
                && a.side == picker.side
                && (!by_player || a.player == picker.player)
            {
                used.extend(a.champion());
            }
        }
        used
    }

    fn excluded(&self, picker: Picker, game: u8, ctx: &Context) -> Excluded {
        let counted = self.counted();
        let mut teammates = HashSet::new();
        let mut opponents = HashSet::new();
        for a in counted.iter().filter(|a| a.game == Some(game)) {
            let Some(c) = a.champion() else {
                continue;
            };
            if a.side != picker.side {
                if !self.rules.champions.mirror {
                    opponents.insert(c);
                }
            } else if self.rules.teams()
                && !self.rules.champions.same_team
                && a.player != picker.player
            {
                teammates.insert(c);
            }
        }
        Excluded {
            banned: self.banned(),
            teammates,
            opponents,
            repeat: self.repeat_for(picker, game, &counted, ctx),
        }
    }

    /// O que `picker` pode escolher para a partida `game` e o que cedeu (ver
    /// o `//!`), sem olhar se é a vez dele (`options`, `team_options`).
    pub fn pick_options(&self, picker: Picker, game: u8, ctx: &Context) -> PickOptions {
        let ex = self.excluded(picker, game, ctx);
        let unbanned: Vec<ChampionId> = ctx
            .champions
            .iter()
            .copied()
            .filter(|c| !ex.banned.contains(c))
            .collect();
        let free: Vec<ChampionId> = unbanned
            .iter()
            .copied()
            .filter(|c| !ex.teammates.contains(c) && !ex.opponents.contains(c))
            .collect();
        let pickable: Vec<ChampionId> = free
            .iter()
            .copied()
            .filter(|c| !ex.repeat.contains(c))
            .collect();
        let (champions, gave_way) = if !pickable.is_empty() {
            (pickable, None)
        } else if !free.is_empty() {
            (free, Some(GaveWay::Rule))
        } else {
            (unbanned, Some(GaveWay::Game))
        };
        PickOptions {
            gave_way: gave_way.filter(|_| !champions.is_empty()),
            champions,
        }
    }

    /// Por que cada campeão do torneio não está nas opções de `picker` para
    /// a partida `game`, sem contar o que cedeu (para a tela); os que estão
    /// livres não aparecem.
    pub fn unavailable(
        &self,
        picker: Picker,
        game: u8,
        ctx: &Context,
    ) -> Vec<(ChampionId, Unavailable)> {
        let ex = self.excluded(picker, game, ctx);
        ctx.champions
            .iter()
            .filter_map(|c| ex.reason(*c).map(|r| (*c, r)))
            .collect()
    }

    /// O erro de um pick fora das opções: `ChampionForbidden` pela regra de
    /// não repetição, `ChampionTaken` se outro já o escolheu para a partida,
    /// `OptionInvalid` no resto (banido, fora do pool).
    pub(super) fn rejection(
        &self,
        picker: Picker,
        game: u8,
        c: ChampionId,
        ctx: &Context,
    ) -> VetoError {
        if !ctx.champions.contains(&c) {
            return VetoError::OptionInvalid;
        }
        match self.excluded(picker, game, ctx).reason(c) {
            Some(Unavailable::Repeat) => VetoError::ChampionForbidden,
            Some(Unavailable::Teammate | Unavailable::Opponent) => VetoError::ChampionTaken,
            Some(Unavailable::Banned) | None => VetoError::OptionInvalid,
        }
    }

    /// Confere um passo do draft do duelo (ban ou pick para `pick_for`). Sem
    /// `ctx` (no `replay`), só o ban.
    pub(super) fn check_draft(
        &self,
        side: Side,
        action: Action,
        pick_for: Option<u8>,
        c: ChampionId,
        ctx: Option<&Context>,
    ) -> Result<(), VetoError> {
        let Some(ctx) = ctx else {
            return if self.banned().contains(&c) {
                Err(VetoError::OptionInvalid)
            } else {
                Ok(())
            };
        };
        match action {
            Action::Ban if self.ban_options(side, ctx).contains(&c) => Ok(()),
            Action::Ban => Err(VetoError::OptionInvalid),
            Action::Pick => {
                let game = pick_for.ok_or(VetoError::WrongAction)?;
                let picker = Picker { side, player: None };
                if self.pick_options(picker, game, ctx).champions.contains(&c) {
                    Ok(())
                } else {
                    Err(self.rejection(picker, game, c, ctx))
                }
            }
            Action::Vote | Action::TeamPick => Err(VetoError::WrongAction),
        }
    }

    /// O que `side` pode banir agora: os sem ban, com a proteção do ban (ver
    /// o `//!`).
    pub(super) fn ban_options(&self, side: Side, ctx: &Context) -> Vec<ChampionId> {
        let banned = self.banned();
        let unbanned: Vec<ChampionId> = ctx
            .champions
            .iter()
            .copied()
            .filter(|c| !banned.contains(c))
            .collect();
        let done = self.get(1).map_or(0, |g| g.split().1.len());
        let opponent_bans = self
            .draft_sequence()
            .iter()
            .skip(done + 1)
            .filter(|s| s.action == Action::Ban && s.side != side)
            .count();
        let needed = opponent_bans + 1 + self.picks_needed(side);
        let sets = self.pickable_sets(side, &unbanned, ctx);
        let at_risk: Vec<&HashSet<ChampionId>> = sets.iter().filter(|s| s.len() < needed).collect();
        if at_risk.is_empty() {
            return unbanned;
        }
        let safe: Vec<ChampionId> = unbanned
            .iter()
            .copied()
            .filter(|c| at_risk.iter().all(|s| !s.contains(c)))
            .collect();
        if safe.is_empty() { unbanned } else { safe }
    }

    /// Campeões que precisam sobrar no conjunto de cada um que escolhe do
    /// lado `side` para ele escolher todas as partidas, no pior caso de
    /// `champions::required_pool`: os das outras partidas (um por partida,
    /// o dele; ou o time todo, um por jogador por partida, com a regra do
    /// time), os dos colegas na mesma partida (sem repetir no time), sem
    /// espelho os dos adversários nela, e o dele.
    fn picks_needed(&self, side: Side) -> usize {
        let size = |s: Side| -> usize {
            if !self.rules.teams() {
                return 1;
            }
            match &self.rosters {
                Some(r) => r.players[index(s)].len().max(1),
                None => usize::from(self.rules.team_size),
            }
        };
        let mine = size(side);
        let mates = if self.rules.champions.same_team {
            0
        } else {
            mine - 1
        };
        let opponents = if self.rules.champions.mirror {
            0
        } else {
            size(other(side))
        };
        let earlier = usize::from(self.best_of).saturating_sub(1);
        let before = match self.rules.no_repeat() {
            None => 0,
            Some(NoRepeatWho::Player) => earlier,
            Some(NoRepeatWho::Team) => earlier * mine,
        };
        before + mates + opponents + 1
    }

    /// Para a proteção do ban: o que cada um que escolhe do lado `side` pode
    /// escolher entre os `unbanned`, pela regra. Pela regra do jogador nos
    /// times, um conjunto por jogador do elenco; no resto, um só, o do lado.
    fn pickable_sets(
        &self,
        side: Side,
        unbanned: &[ChampionId],
        ctx: &Context,
    ) -> Vec<HashSet<ChampionId>> {
        let counted = self.counted();
        let without = |picker: Picker| -> HashSet<ChampionId> {
            let forbidden = self.repeat_for(picker, 0, &counted, ctx);
            unbanned
                .iter()
                .copied()
                .filter(|c| !forbidden.contains(c))
                .collect()
        };
        let roster = self.roster(side);
        let by_player = self.rules.teams() && self.rules.no_repeat() == Some(NoRepeatWho::Player);
        if !by_player || roster.is_empty() {
            return vec![without(Picker { side, player: None })];
        }
        roster
            .iter()
            .map(|p| {
                without(Picker {
                    side,
                    player: Some(*p),
                })
            })
            .collect()
    }
}
