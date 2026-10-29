//! Veto de mapas e ban/pick de campeões de uma série, ao vivo, em turnos.
//!
//! O estado nunca é guardado: sai do `replay` das regras do torneio, da
//! melhor de N da partida e do histórico de eventos (começo, início de cada
//! jogo, passos e desfazer). Quem grava o histórico (a API) é quem sabe a
//! hora: o cronômetro e o "pronto" dos jogadores ficam lá.
//!
//! Regras:
//! - **Modo do mapa**, por torneio: `Veto` (ban/pick alternado da série
//!   inteira antes do primeiro jogo, com o decisivo: `map_sequence`), `Vote`
//!   (a cada jogo, até 3 mapas sorteados do pool, sem os já jogados na série;
//!   cada lado vota num; votos iguais, é esse; diferentes, sorteio entre os
//!   dois), `Random` (o mapa do jogo é sorteado quando ele começa, sem
//!   repetir) e `Off` (os jogadores combinam fora).
//! - **Campeões**, por mapa, se a fase está ligada: `champion_sequence`
//!   (ban de um lado, ban do outro, tantas vezes quantos os bans por mapa;
//!   pick de um, pick do outro), alternado e aberto. Nos times (TDM,
//!   `team_size` > 1), os bans são os mesmos (pelo capitão, na API) e no
//!   lugar dos dois picks há uma **escolha simultânea** (`Action::TeamPick`,
//!   `team.rs`): cada jogador do elenco de cada lado (`set_rosters`, quem
//!   está no time agora) escolhe o próprio campeão, sem vez, e o jogo
//!   acaba quando todos escolheram ou quando foi enviado (sem o elenco
//!   informado, a vez é `Turn::RostersUnknown`). O ban vale para os dois
//!   naquele mapa. Começa quem não escolheu o mapa; no decisivo, quem começou
//!   o veto (o seed melhor); fora do modo veto, o seed melhor no primeiro
//!   jogo e, nos seguintes, quem perdeu o anterior (gravado no evento
//!   `Game`, para a correção de um resultado não mudar o veto já feito).
//! - **Não repetição** (`Context::no_repeat`): o pick exclui os campeões que
//!   o lado já usou nos jogos anteriores da série e os de fora dela que a API
//!   passa (`Context::used`, do escopo da regra). Nos times, pela regra do
//!   jogador, o que ele mesmo usou (`Context::used_by` de fora da série);
//!   pela do time, o que qualquer um do time usou; e, sem repetir no time
//!   (`team_duplicates` falso), o que um colega já escolheu no mapa. Se não sobrar nenhum, a
//!   regra cede (qualquer campeão sem ban), para o veto nunca travar. Um ban
//!   também não pode deixar quem bane sem campeão para escolher: com tão
//!   poucos que os bans que o adversário ainda tem os esgotariam, quem bane
//!   só bane os que ele mesmo não poderia escolher (`Veto::options`; nos
//!   times, conta o que sobra para cada jogador do elenco).
//! - Os passos do jogo em andamento se desfazem do último para o primeiro
//!   (`Event::Undo`); jogo que já passou não se desfaz.
//! - Os sorteios (mapas oferecidos, desempate, mapa aleatório e a escolha de
//!   quem deixou o tempo acabar, `random_choice`) saem da seed gravada no
//!   começo: o mesmo histórico dá sempre o mesmo resultado.
//! - No `replay`, a regra de não repetição e o pool de campeões não são
//!   conferidos de novo (valeram quando o passo foi gravado); a vez, o tipo
//!   do passo e o que já saiu, sim. Na escolha simultânea, o elenco também
//!   não: quem escolheu num jogo pode ter saído do time depois.

mod draw;
mod sequence;
mod team;
#[cfg(test)]
mod tests;

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::champions::NoRepeatWho;
use crate::core::{ChampionId, MapId, PlayerId};
use crate::series::Side;
use draw::Purpose;

pub use sequence::{Action, champion_sequence, map_sequence};
pub use team::{GaveWay, Rosters, TeamOptions};

/// Mapas oferecidos em cada votação (como no próprio Quake Champions).
pub const VOTE_OPTIONS: usize = 3;

/// Como sai o mapa de cada jogo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MapMode {
    Veto,
    Vote,
    Random,
    Off,
}

/// O que o torneio define para o veto das séries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VetoRules {
    pub map_mode: MapMode,
    /// Pool do torneio, na ordem configurada (fora do modo `Off`).
    pub pool: Vec<MapId>,
    /// Ban/pick de campeões a cada mapa.
    pub champion_phase: bool,
    /// Bans de campeão de cada lado por mapa.
    pub champion_bans: u8,
    /// Jogadores por lado: 1 no duelo; nos times, a escolha de campeões é
    /// simultânea, um por jogador.
    pub team_size: u8,
    /// Nos times: dois do mesmo time podem ter o mesmo campeão no mapa.
    pub team_duplicates: bool,
}

impl VetoRules {
    /// Há alguma fase antes dos jogos (mapa ou campeões).
    pub fn has_phases(&self) -> bool {
        self.map_mode != MapMode::Off || self.champion_phase
    }

    /// Nos times, a escolha de campeões é simultânea.
    pub const fn teams(&self) -> bool {
        self.team_size > 1
    }

    /// O pool aguenta séries de até `longest` mapas: sem mapa repetido e,
    /// fora do modo `Off`, com pelo menos um mapa por jogo. Erros:
    /// `PoolRepeated`, `PoolTooSmall`.
    pub fn validate(&self, longest: u8) -> Result<(), VetoError> {
        let mut seen = HashSet::new();
        for map in &self.pool {
            if !seen.insert(*map) {
                return Err(VetoError::PoolRepeated(*map));
            }
        }
        if self.map_mode != MapMode::Off && self.pool.len() < usize::from(longest) {
            return Err(VetoError::PoolTooSmall {
                pool: self.pool.len(),
                best_of: longest,
            });
        }
        Ok(())
    }
}

/// O que um passo escolhe.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Choice {
    Map(MapId),
    Champion(ChampionId),
}

/// Um evento do histórico do veto de uma série, na ordem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// O veto começa (os dois prontos, ou o organizador forçou): `first` é o
    /// seed melhor, que começa o veto de mapas e os campeões do primeiro
    /// jogo fora do modo veto; `seed`, a dos sorteios da série. Abre o jogo 1.
    Begin { seed: u64, first: Side },
    /// O jogo `game` (2 em diante) começa: o anterior foi enviado. `starter`
    /// começa os campeões dele (`Veto::next_starter`); `previous`, o mapa em
    /// que o anterior foi enviado, quando há: é ele que sai do pool da
    /// votação e do sorteio dos jogos seguintes (o organizador pode ter
    /// lançado o anterior sem o veto, ou em outro mapa).
    Game {
        game: u8,
        starter: Side,
        previous: Option<MapId>,
    },
    /// Um passo do jogo em andamento.
    Step {
        side: Side,
        action: Action,
        choice: Choice,
    },
    /// Nos times: `player`, do lado `side`, escolhe o próprio campeão na
    /// escolha simultânea do jogo em andamento.
    TeamPick {
        side: Side,
        player: PlayerId,
        champion: ChampionId,
    },
    /// Desfaz o último passo do jogo em andamento.
    Undo,
}

/// Um passo que vale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Applied {
    pub side: Side,
    pub action: Action,
    pub choice: Choice,
    /// Na escolha simultânea dos times, o jogador.
    pub player: Option<PlayerId>,
}

/// Sobre o que é o passo esperado.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    Map,
    Champion,
}

/// O que o veto espera agora.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Turn {
    /// Ainda não começou.
    NotBegun,
    /// O jogo `game` espera `action` sobre `target` de `sides` (os dois na
    /// votação, enquanto nenhum votou; na escolha simultânea dos times, os
    /// lados em que falta alguém: `Veto::missing`).
    Act {
        game: u8,
        action: Action,
        target: Target,
        sides: Vec<Side>,
    },
    /// O veto do jogo `game` acabou: espera o envio dele.
    Done { game: u8 },
    /// Nos times, o jogo `game` chegou à escolha simultânea, mas o elenco de
    /// agora não foi informado (`Veto::set_rosters`): sem ele, não se sabe
    /// quem falta nem se o jogo já foi enviado. O `replay` sozinho para
    /// aqui; a API sempre informa.
    RostersUnknown { game: u8 },
}

/// O que as opções de campeão precisam saber de fora da série.
#[derive(Clone, Copy, Debug)]
pub struct Context<'a> {
    /// Os campeões do torneio, na ordem em que aparecem.
    pub champions: &'a [ChampionId],
    /// A regra de não repetição, quando vale (no duelo, `Player` e `Team`
    /// dão no mesmo).
    pub no_repeat: Option<NoRepeatWho>,
    /// O que cada lado (`[a, b]`) já usou fora desta série, no escopo da
    /// regra (vazio no escopo da partida). Nos times, pela regra do time.
    pub used: [&'a [ChampionId]; 2],
    /// Nos times, pela regra do jogador: o que cada um já usou fora desta
    /// série, no escopo da regra.
    pub used_by: &'a [(PlayerId, ChampionId)],
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum VetoError {
    #[error("o pool tem {pool} mapas para uma série melhor de {best_of}")]
    PoolTooSmall { pool: usize, best_of: u8 },
    #[error("mapa repetido no pool: {0:?}")]
    PoolRepeated(MapId),
    #[error("o veto já começou")]
    AlreadyBegun,
    #[error("o veto ainda não começou")]
    NotBegun,
    #[error("o próximo jogo do veto é o {next}")]
    GameOutOfOrder { next: u8 },
    #[error("o veto deste jogo já acabou")]
    Finished,
    #[error("não é a vez deste lado")]
    NotYourTurn,
    #[error("o passo esperado é outro")]
    WrongAction,
    #[error("escolha fora das opções")]
    OptionInvalid,
    #[error("campeão já usado, pela regra de não repetição")]
    ChampionForbidden,
    #[error("nada para desfazer neste jogo")]
    NothingToUndo,
    #[error("este jogador já escolheu o campeão deste mapa")]
    AlreadyPicked,
    #[error("o elenco de agora não foi informado")]
    RostersUnknown,
    #[error("jogador repetido no elenco: {0:?}")]
    RosterRepeated(PlayerId),
    #[error("elenco com {got} jogadores, o máximo é {max}")]
    RosterTooLarge { max: u8, got: usize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Begin {
    seed: u64,
    first: Side,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GameVeto {
    number: u8,
    /// Do evento `Game`; `None` no primeiro jogo.
    starter: Option<Side>,
    /// Do evento `Game`: o mapa em que o jogo anterior foi enviado.
    previous: Option<MapId>,
    /// Os passos de mapa primeiro, depois os de campeão.
    steps: Vec<Applied>,
}

impl GameVeto {
    fn split(&self) -> (&[Applied], &[Applied]) {
        let maps = self
            .steps
            .iter()
            .take_while(|s| matches!(s.choice, Choice::Map(_)))
            .count();
        self.steps.split_at(maps)
    }
}

const fn other(side: Side) -> Side {
    match side {
        Side::A => Side::B,
        Side::B => Side::A,
    }
}

/// O lado do passo `i` (a partir de 0) numa sequência que `first` começa.
fn alternate(first: Side, i: usize) -> Side {
    if i.is_multiple_of(2) {
        first
    } else {
        other(first)
    }
}

const fn index(side: Side) -> usize {
    match side {
        Side::A => 0,
        Side::B => 1,
    }
}

/// O veto de uma série. Só `Serialize`-ável por fora do jeito que a API
/// monta: para reidratar, use `replay`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Veto {
    rules: VetoRules,
    best_of: u8,
    begin: Option<Begin>,
    games: Vec<GameVeto>,
    /// Eventos aplicados: separa os sorteios de tempo esgotado.
    events: u32,
    /// Nos times, o que só a API sabe agora (`set_rosters`): quem escolhe
    /// campeão por lado e o que já foi enviado. `None` até ser informado.
    rosters: Option<Rosters>,
}

/// Onde o jogo em andamento está, sem olhar o elenco.
enum Phase {
    /// Fora da escolha simultânea: a vez é esta.
    Turn(Turn),
    /// Nos times, o jogo chegou à escolha simultânea (o veto de mapa dele e
    /// os bans acabaram): quem falta depende do elenco.
    TeamPick(u8),
}

impl Veto {
    pub fn new(rules: VetoRules, best_of: u8) -> Self {
        Self {
            rules,
            best_of,
            begin: None,
            games: Vec::new(),
            events: 0,
            rosters: None,
        }
    }

    /// Informa o que o histórico não diz (ver `Rosters`). Vale para o jogo
    /// em andamento; o `replay` não depende dele. Erros: `RosterRepeated` (o
    /// mesmo jogador duas vezes, num lado ou nos dois) e `RosterTooLarge`
    /// (mais que `team_size` num lado).
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

    /// Refaz o veto aplicando `events` em ordem, sem conferir a regra de não
    /// repetição nem o pool de campeões (ver o `//!`).
    pub fn replay(
        rules: VetoRules,
        best_of: u8,
        events: impl IntoIterator<Item = Event>,
    ) -> Result<Self, VetoError> {
        let mut veto = Self::new(rules, best_of);
        for event in events {
            veto.apply(event, None)?;
        }
        Ok(veto)
    }

    /// Aplica um evento. Com `ctx`, um passo de campeão também passa pela
    /// regra de não repetição e pelo pool (`options`, `team_options`), e a
    /// escolha simultânea, pelo elenco. Erros: `AlreadyBegun`,
    /// `PoolTooSmall` (no começo), `NotBegun`, `GameOutOfOrder`, `Finished`,
    /// `NotYourTurn`, `WrongAction`, `OptionInvalid`, `ChampionForbidden`,
    /// `AlreadyPicked`, `RostersUnknown` (escolha simultânea com `ctx` sem o
    /// elenco informado) e `NothingToUndo`.
    pub fn apply(&mut self, event: Event, ctx: Option<&Context>) -> Result<(), VetoError> {
        match event {
            Event::Begin { seed, first } => {
                if self.begin.is_some() {
                    return Err(VetoError::AlreadyBegun);
                }
                if self.best_of == 0 {
                    return Err(VetoError::GameOutOfOrder { next: 1 });
                }
                if self.rules.map_mode != MapMode::Off {
                    map_sequence(self.rules.pool.len(), self.best_of)?;
                }
                self.begin = Some(Begin { seed, first });
                self.games.push(GameVeto {
                    number: 1,
                    starter: None,
                    previous: None,
                    steps: Vec::new(),
                });
            }
            Event::Game {
                game,
                starter,
                previous,
            } => {
                let next = self.games.len().saturating_add(1);
                if self.begin.is_none() {
                    return Err(VetoError::NotBegun);
                }
                if usize::from(game) != next || game > self.best_of {
                    let next = u8::try_from(next).unwrap_or(u8::MAX);
                    return Err(VetoError::GameOutOfOrder { next });
                }
                self.games.push(GameVeto {
                    number: game,
                    starter: Some(starter),
                    previous,
                    steps: Vec::new(),
                });
            }
            Event::Step {
                side,
                action,
                choice,
            } => self.step(side, action, choice, ctx)?,
            Event::TeamPick {
                side,
                player,
                champion,
            } => self.team_step(side, player, champion, ctx)?,
            Event::Undo => {
                let game = self.games.last_mut().ok_or(VetoError::NotBegun)?;
                game.steps.pop().ok_or(VetoError::NothingToUndo)?;
            }
        }
        self.events += 1;
        Ok(())
    }

    fn step(
        &mut self,
        side: Side,
        action: Action,
        choice: Choice,
        ctx: Option<&Context>,
    ) -> Result<(), VetoError> {
        if action == Action::TeamPick {
            // A escolha simultânea é por jogador: `Event::TeamPick`.
            return Err(VetoError::WrongAction);
        }
        let (game, expected, target, sides) = match self.turn() {
            Turn::NotBegun => return Err(VetoError::NotBegun),
            Turn::Done { .. } => return Err(VetoError::Finished),
            // O jogo está na escolha simultânea, que é por jogador.
            Turn::RostersUnknown { .. } => return Err(VetoError::WrongAction),
            Turn::Act {
                game,
                action,
                target,
                sides,
            } => (game, action, target, sides),
        };
        if !sides.contains(&side) {
            return Err(VetoError::NotYourTurn);
        }
        if action != expected {
            return Err(VetoError::WrongAction);
        }
        let valid = match (target, choice) {
            (Target::Map, Choice::Map(map)) if action == Action::Vote => {
                self.offered(game).contains(&map)
            }
            (Target::Map, Choice::Map(map)) => self.remaining_maps().contains(&map),
            (Target::Champion, Choice::Champion(c)) => match ctx {
                Some(ctx) => {
                    if !self.options(side, ctx).contains(&choice) {
                        let forbidden = action == Action::Pick
                            && ctx.champions.contains(&c)
                            && !self.banned(game).contains(&c)
                            && self.forbidden(side, ctx).contains(&c);
                        return Err(if forbidden {
                            VetoError::ChampionForbidden
                        } else {
                            VetoError::OptionInvalid
                        });
                    }
                    true
                }
                None => !self.banned(game).contains(&c),
            },
            _ => false,
        };
        if !valid {
            return Err(VetoError::OptionInvalid);
        }
        let current = self.games.last_mut().ok_or(VetoError::NotBegun)?;
        current.steps.push(Applied {
            side,
            action,
            choice,
            player: None,
        });
        Ok(())
    }

    pub fn rules(&self) -> &VetoRules {
        &self.rules
    }

    pub fn best_of(&self) -> u8 {
        self.best_of
    }

    pub fn begun(&self) -> bool {
        self.begin.is_some()
    }

    /// Quem começou o veto (o seed melhor).
    pub fn first(&self) -> Option<Side> {
        self.begin.map(|b| b.first)
    }

    /// A seed dos sorteios da série.
    pub fn seed(&self) -> Option<u64> {
        self.begin.map(|b| b.seed)
    }

    /// O jogo em andamento (o último que começou).
    pub fn game(&self) -> Option<u8> {
        self.games.last().map(|g| g.number)
    }

    fn get(&self, game: u8) -> Option<&GameVeto> {
        self.games.get(usize::from(game).checked_sub(1)?)
    }

    pub fn turn(&self) -> Turn {
        match self.phase() {
            Phase::Turn(turn) => turn,
            Phase::TeamPick(game) => self.team_turn(game),
        }
    }

    /// A vez sem o elenco: na escolha simultânea dos times, só o jogo.
    fn phase(&self) -> Phase {
        let (Some(begin), Some(current)) = (self.begin, self.games.last()) else {
            return Phase::Turn(Turn::NotBegun);
        };
        let game = current.number;
        let (maps, champions) = current.split();
        match self.rules.map_mode {
            MapMode::Veto if game == 1 => {
                let sequence = self.map_sequence();
                if let Some(&action) = sequence.get(maps.len()) {
                    return Phase::Turn(Turn::Act {
                        game,
                        action,
                        target: Target::Map,
                        sides: vec![alternate(begin.first, maps.len())],
                    });
                }
            }
            MapMode::Vote if self.offered(game).len() > 1 => {
                let missing: Vec<Side> = [Side::A, Side::B]
                    .into_iter()
                    .filter(|s| !maps.iter().any(|m| m.side == *s))
                    .collect();
                if !missing.is_empty() {
                    return Phase::Turn(Turn::Act {
                        game,
                        action: Action::Vote,
                        target: Target::Map,
                        sides: missing,
                    });
                }
            }
            _ => {}
        }
        if !self.rules.champion_phase {
            return Phase::Turn(Turn::Done { game });
        }
        let starter = self.starter(game).unwrap_or(begin.first);
        if self.rules.teams() && champions.len() >= self.champion_bans() {
            return Phase::TeamPick(game);
        }
        let sequence = self.champion_sequence();
        let Some(&action) = sequence.get(champions.len()) else {
            return Phase::Turn(Turn::Done { game });
        };
        Phase::Turn(Turn::Act {
            game,
            action,
            target: Target::Champion,
            sides: vec![alternate(starter, champions.len())],
        })
    }

    fn map_sequence(&self) -> Vec<Action> {
        map_sequence(self.rules.pool.len(), self.best_of).unwrap_or_default()
    }

    fn champion_sequence(&self) -> Vec<Action> {
        if self.rules.champion_phase {
            champion_sequence(self.rules.champion_bans)
        } else {
            Vec::new()
        }
    }

    /// Passos de ban de campeão por mapa (os dois lados).
    fn champion_bans(&self) -> usize {
        usize::from(self.rules.champion_bans) * 2
    }

    /// Há passo para desfazer no jogo em andamento.
    pub fn can_undo(&self) -> bool {
        self.games.last().is_some_and(|g| !g.steps.is_empty())
    }

    // Veto de mapas (modo veto).

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

    /// Quem escolheu o mapa do jogo `game` no veto; `None` no decisivo e fora
    /// do modo veto.
    pub fn picker(&self, game: u8) -> Option<Side> {
        if self.rules.map_mode != MapMode::Veto || game == 0 {
            return None;
        }
        self.veto_picks()
            .get(usize::from(game) - 1)
            .map(|(side, _)| *side)
    }

    // Mapa de cada jogo.

    /// O mapa do jogo `game`, quando já saiu: do veto (o pick aparece assim
    /// que é feito, já que o veto é aberto; o decisivo, com o veto completo),
    /// da votação ou do sorteio. `None` no modo `Off`.
    pub fn map_of(&self, game: u8) -> Option<MapId> {
        if game == 0 || game > self.best_of {
            return None;
        }
        let before = self.maps_before(game);
        self.map_with(game, &before)
    }

    /// O mapa de cada jogo antes de `game`, numa passada só (o de um jogo
    /// depende dos que saíram antes dele).
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

    /// Os mapas do pool que ainda não saíram nos jogos `before`: o mapa em
    /// que cada um foi enviado (gravado no `Game` do seguinte) ou, sem ele,
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

    /// Os mapas oferecidos na votação do jogo `game` (modo votação, jogo já
    /// começado), na ordem do pool: até `VOTE_OPTIONS` sorteados dos que
    /// ainda não saíram na série.
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
            VOTE_OPTIONS,
        )
        .into_iter()
        .map(|i| available[i])
        .collect()
    }

    /// O voto de cada lado (`[a, b]`) no jogo `game`.
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

    // Campeões.

    /// Quem começa os campeões do jogo `game` (ver o `//!`).
    pub fn starter(&self, game: u8) -> Option<Side> {
        let begin = self.begin?;
        let g = self.get(game)?;
        if self.map_veto_done() {
            return Some(self.picker(game).map_or(begin.first, other));
        }
        Some(g.starter.unwrap_or(begin.first))
    }

    /// Quem começa os campeões do próximo jogo, para o evento `Game`: quem
    /// não escolheu o mapa dele no veto (no decisivo, quem começou o veto);
    /// fora do veto, `loser`, quem perdeu o jogo em andamento.
    pub fn next_starter(&self, loser: Side) -> Side {
        let next = self.game().unwrap_or(0).saturating_add(1);
        match (self.map_veto_done(), self.first()) {
            (true, Some(first)) => self.picker(next).map_or(first, other),
            _ => loser,
        }
    }

    /// Os passos de campeão do jogo `game`, na ordem.
    pub fn champion_steps(&self, game: u8) -> &[Applied] {
        self.get(game).map_or(&[], |g| g.split().1)
    }

    fn banned(&self, game: u8) -> Vec<ChampionId> {
        self.champion_steps(game)
            .iter()
            .filter(|s| s.action == Action::Ban)
            .filter_map(|s| match s.choice {
                Choice::Champion(c) => Some(c),
                Choice::Map(_) => None,
            })
            .collect()
    }

    /// O campeão de cada lado (`[a, b]`) no jogo `game`.
    pub fn picks(&self, game: u8) -> [Option<ChampionId>; 2] {
        let mut picks = [None, None];
        for s in self.champion_steps(game) {
            if let (Action::Pick, Choice::Champion(c)) = (s.action, s.choice) {
                picks[index(s.side)] = Some(c);
            }
        }
        picks
    }

    /// Os campeões que a regra de não repetição tira de `side` no jogo em
    /// andamento: os de fora da série (`ctx.used`) e os que ele escolheu nos
    /// jogos anteriores dela.
    pub fn forbidden(&self, side: Side, ctx: &Context) -> HashSet<ChampionId> {
        if ctx.no_repeat.is_none() {
            return HashSet::new();
        }
        if self.rules.teams() {
            return self.team_forbidden(side, ctx);
        }
        let current = self.game().unwrap_or(0);
        let mut used: HashSet<ChampionId> = ctx.used[index(side)].iter().copied().collect();
        used.extend((1..current).filter_map(|g| self.picks(g)[index(side)]));
        used
    }

    /// O que `side` pode escolher agora (vazio se não é a vez dele, e na
    /// escolha simultânea dos times, que é por jogador: `team_options`): no
    /// veto de mapas, os que sobraram; na votação, os oferecidos; nos
    /// campeões, os sem ban, menos os da regra no pick e com a proteção do
    /// ban (ver o `//!`): quem bane precisa que sobrem, depois dos bans que
    /// o adversário ainda tem, `picks_needed` campeões que ele pode escolher.
    /// Nos times, conta o que sobra para cada jogador do elenco (pela regra
    /// do jogador, cada um tem o seu): se algum ficaria sem, o ban fica nos
    /// que nenhum desses pode escolher; se não há nenhum assim, a proteção
    /// cede, como a regra.
    pub fn options(&self, side: Side, ctx: &Context) -> Vec<Choice> {
        let Turn::Act {
            game,
            action,
            target,
            sides,
        } = self.turn()
        else {
            return Vec::new();
        };
        if !sides.contains(&side) {
            return Vec::new();
        }
        if target == Target::Map {
            let maps = if action == Action::Vote {
                self.offered(game)
            } else {
                self.remaining_maps()
            };
            return maps.into_iter().map(Choice::Map).collect();
        }
        let banned = self.banned(game);
        let unbanned: Vec<ChampionId> = ctx
            .champions
            .iter()
            .copied()
            .filter(|c| !banned.contains(c))
            .collect();
        let forbidden = self.forbidden(side, ctx);
        let pickable: Vec<ChampionId> = unbanned
            .iter()
            .copied()
            .filter(|c| !forbidden.contains(c))
            .collect();
        let chosen = match action {
            Action::TeamPick => Vec::new(),
            Action::Pick if pickable.is_empty() => unbanned,
            Action::Pick | Action::Vote => pickable,
            Action::Ban => {
                let done = self.champion_steps(game).len();
                let starter = self.starter(game).unwrap_or(side);
                let opponent_bans = self
                    .champion_sequence()
                    .iter()
                    .enumerate()
                    .skip(done + 1)
                    .filter(|(i, a)| **a == Action::Ban && alternate(starter, *i) != side)
                    .count();
                let needed = opponent_bans + 1 + self.picks_needed(side);
                let pickers = self.pickable_sets(side, &unbanned, ctx);
                let at_risk: Vec<&HashSet<ChampionId>> =
                    pickers.iter().filter(|s| s.len() < needed).collect();
                if at_risk.is_empty() {
                    unbanned
                } else {
                    let safe: Vec<ChampionId> = unbanned
                        .iter()
                        .copied()
                        .filter(|c| at_risk.iter().all(|s| !s.contains(c)))
                        .collect();
                    if safe.is_empty() { unbanned } else { safe }
                }
            }
        };
        chosen.into_iter().map(Choice::Champion).collect()
    }

    /// A escolha de `side` quando o tempo do passo acaba: uma das `options`,
    /// sorteada pela seed da série e pela posição no histórico (o mesmo
    /// histórico dá a mesma escolha). `None` se não é a vez dele.
    pub fn random_choice(&self, side: Side, ctx: &Context) -> Option<Choice> {
        let options = self.options(side, ctx);
        if options.is_empty() {
            return None;
        }
        let seed = self.begin?.seed;
        let game = self.game()?;
        let i = draw::index(
            seed,
            Purpose::Timeout,
            game,
            self.events * 2 + index(side) as u32,
            options.len(),
        );
        options.get(i).copied()
    }
}
