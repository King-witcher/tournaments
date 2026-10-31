//! O mapa de cada partida: o veto de mapas da série (modo veto), a votação a
//! cada partida e o sorteio (ver o `//!` do módulo).

use super::{
    Action, Applied, Begin, Choice, MapMode, Target, Turn, Veto, VetoError, alternate, draw,
    draw::Purpose, index, map_sequence,
};
use crate::core::MapId;
use crate::series::Side;

impl Veto {
    /// A vez do mapa da partida `game`, com os passos de mapa `maps` dela
    /// feitos; `None` quando o mapa não espera mais nada.
    pub(super) fn map_turn(&self, begin: Begin, game: u8, maps: &[Applied]) -> Option<Turn> {
        match self.rules.map_mode {
            MapMode::Veto if game == 1 => {
                let action = *self.map_sequence().get(maps.len())?;
                Some(Turn::Act {
                    game,
                    action,
                    target: Target::Map,
                    sides: vec![alternate(begin.first, maps.len())],
                    pick_for: None,
                })
            }
            MapMode::Vote if self.offered(game).len() > 1 => {
                let missing: Vec<Side> = [Side::A, Side::B]
                    .into_iter()
                    .filter(|s| !maps.iter().any(|m| m.side == *s))
                    .collect();
                (!missing.is_empty()).then_some(Turn::Act {
                    game,
                    action: Action::Vote,
                    target: Target::Map,
                    sides: missing,
                    pick_for: None,
                })
            }
            _ => None,
        }
    }

    /// O mapa do passo de mapa vale: na votação, um dos oferecidos; no veto,
    /// um dos que sobraram. Erro: `OptionInvalid`.
    pub(super) fn check_map(&self, game: u8, action: Action, map: MapId) -> Result<(), VetoError> {
        let valid = if action == Action::Vote {
            self.offered(game).contains(&map)
        } else {
            self.remaining_maps().contains(&map)
        };
        if valid {
            Ok(())
        } else {
            Err(VetoError::OptionInvalid)
        }
    }

    /// As opções do passo de mapa da partida `game`.
    pub(super) fn map_options(&self, game: u8, action: Action) -> Vec<Choice> {
        let maps = if action == Action::Vote {
            self.offered(game)
        } else {
            self.remaining_maps()
        };
        maps.into_iter().map(Choice::Map).collect()
    }

    fn map_sequence(&self) -> Vec<Action> {
        map_sequence(self.rules.pool.len(), self.best_of).unwrap_or_default()
    }

    /// Os passos do veto de mapas da série (modo veto), na ordem.
    pub fn map_steps(&self) -> &[Applied] {
        match (self.rules.map_mode, self.games.first()) {
            (MapMode::Veto, Some(first)) => first.split().0,
            _ => &[],
        }
    }

    fn map_veto_done(&self) -> bool {
        self.rules.map_mode == MapMode::Veto
            && self.begun()
            && self.map_steps().len() == self.map_sequence().len()
    }

    /// Os mapas do pool que ainda não saíram no veto de mapas.
    pub fn remaining_maps(&self) -> Vec<MapId> {
        let used: Vec<MapId> = self
            .map_steps()
            .iter()
            .filter_map(|s| match s.choice {
                Choice::Map(m) => Some(m),
                Choice::Champion(_) => None,
            })
            .collect();
        self.rules
            .pool
            .iter()
            .copied()
            .filter(|m| !used.contains(m))
            .collect()
    }

    /// O decisivo (o mapa que sobrou), com o veto de mapas completo.
    pub fn decider(&self) -> Option<MapId> {
        if !self.map_veto_done() {
            return None;
        }
        self.remaining_maps().first().copied()
    }

    fn veto_picks(&self) -> Vec<(Side, MapId)> {
        self.map_steps()
            .iter()
            .filter(|s| s.action == Action::Pick)
            .filter_map(|s| match s.choice {
                Choice::Map(m) => Some((s.side, m)),
                Choice::Champion(_) => None,
            })
            .collect()
    }

    /// Quem escolheu o mapa da partida `game` no veto; `None` no decisivo e
    /// fora do modo veto.
    pub fn picker(&self, game: u8) -> Option<Side> {
        if self.rules.map_mode != MapMode::Veto || game == 0 {
            return None;
        }
        self.veto_picks()
            .get(usize::from(game) - 1)
            .map(|(side, _)| *side)
    }

    /// O mapa da partida `game`, quando já saiu: do veto (o pick aparece
    /// assim que é feito, já que o veto é aberto; o decisivo, com o veto
    /// completo), da votação ou do sorteio. `None` no modo `Off`.
    pub fn map_of(&self, game: u8) -> Option<MapId> {
        if game == 0 || game > self.best_of {
            return None;
        }
        let before = self.maps_before(game);
        self.map_with(game, &before)
    }

    /// O mapa de cada partida antes de `game`, numa passada só (o de uma
    /// partida depende dos que saíram antes dela).
    fn maps_before(&self, game: u8) -> Vec<Option<MapId>> {
        let mut maps = Vec::new();
        for g in 1..game.min(self.best_of.saturating_add(1)) {
            let m = self.map_with(g, &maps);
            maps.push(m);
        }
        maps
    }

    fn map_with(&self, game: u8, before: &[Option<MapId>]) -> Option<MapId> {
        match self.rules.map_mode {
            MapMode::Off => None,
            MapMode::Veto if game == self.best_of => self.decider(),
            MapMode::Veto => self
                .veto_picks()
                .get(usize::from(game).checked_sub(1)?)
                .map(|(_, m)| *m),
            MapMode::Vote => {
                let offered = self.offered_with(game, before);
                if offered.len() == 1 {
                    return offered.first().copied();
                }
                let [Some(a), Some(b)] = self.votes(game) else {
                    return None;
                };
                if a == b {
                    return Some(a);
                }
                let seed = self.begin?.seed;
                Some(if draw::index(seed, Purpose::Flip, game, 0, 2) == 0 {
                    a
                } else {
                    b
                })
            }
            MapMode::Random => {
                self.get(game)?;
                let available = self.available(before);
                let seed = self.begin?.seed;
                let i = draw::index(seed, Purpose::Draw, game, 0, available.len());
                available.get(i).copied()
            }
        }
    }

    /// Os mapas do pool que ainda não saíram nas partidas `before`: o mapa em
    /// que cada uma foi enviada (gravado no `Game` da seguinte) ou, sem ele,
    /// o que o veto deu.
    fn available(&self, before: &[Option<MapId>]) -> Vec<MapId> {
        let played: Vec<MapId> = before
            .iter()
            .enumerate()
            .filter_map(|(i, m)| self.games.get(i + 1).and_then(|g| g.previous).or(*m))
            .collect();
        self.rules
            .pool
            .iter()
            .copied()
            .filter(|m| !played.contains(m))
            .collect()
    }

    /// Os mapas oferecidos na votação da partida `game` (modo votação,
    /// partida já começada), na ordem do pool: até `VOTE_OPTIONS` sorteados
    /// dos que ainda não saíram na série.
    pub fn offered(&self, game: u8) -> Vec<MapId> {
        let before = self.maps_before(game);
        self.offered_with(game, &before)
    }

    fn offered_with(&self, game: u8, before: &[Option<MapId>]) -> Vec<MapId> {
        let (Some(begin), Some(_)) = (self.begin, self.get(game)) else {
            return Vec::new();
        };
        if self.rules.map_mode != MapMode::Vote {
            return Vec::new();
        }
        let available = self.available(before);
        draw::choose(
            begin.seed,
            Purpose::Offer,
            game,
            available.len(),
            super::VOTE_OPTIONS,
        )
        .into_iter()
        .map(|i| available[i])
        .collect()
    }

    /// O voto de cada lado (`[a, b]`) na partida `game`.
    pub fn votes(&self, game: u8) -> [Option<MapId>; 2] {
        let mut votes = [None, None];
        if let Some(g) = self.get(game) {
            for s in g.split().0.iter().filter(|s| s.action == Action::Vote) {
                if let Choice::Map(m) = s.choice {
                    votes[index(s.side)] = Some(m);
                }
            }
        }
        votes
    }
}
