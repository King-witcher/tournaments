//! Veto de mapas e draft de campeões de uma série, ao vivo, em turnos.
//!
//! O estado nunca é guardado: sai do `replay` das regras do torneio, da
//! melhor de N da série e do histórico de eventos (começo, início de cada
//! partida, passos e desfazer). Quem grava o histórico (a API) é quem sabe a
//! hora: o cronômetro e o "pronto" dos jogadores ficam lá.
//!
//! Regras:
//! - **Modo do mapa**, por torneio (`maps.rs`): `Veto` (ban/pick alternado
//!   da série inteira antes da partida 1, com o decisivo: `map_sequence`),
//!   `Vote` (a cada partida, até 3 mapas sorteados do pool, sem os já
//!   jogados na série; cada lado vota num; votos iguais, é esse; diferentes,
//!   sorteio entre os dois), `Random` (o mapa da partida é sorteado quando
//!   ela começa, sem repetir) e `Off` (os jogadores combinam fora).
//! - **Draft de campeões** da série inteira, antes da partida 1, depois do
//!   mapa dela (`draft.rs`, `draft_sequence`): os bans da série, alternados
//!   a partir do seed pior, valem para os dois lados em todas as partidas.
//!   No duelo, depois, um pick de cada lado por partida, na ordem das
//!   partidas, alternando quem escolhe primeiro. Nos times (TDM,
//!   `team_size` > 1), no lugar dos picks há uma **janela simultânea**
//!   (`team.rs`): cada jogador do elenco de cada lado (`set_rosters`)
//!   escolhe o próprio campeão de cada partida, em qualquer ordem. Ninguém
//!   volta ao veto entre as partidas, salvo para votar o mapa (modo votação)
//!   ou, nos times, quem entrou no elenco depois do draft.
//! - **O que pode repetir** (`VetoRules::champions`): adversários na mesma
//!   partida (`mirror`; sem ele, fica com o campeão quem escolhe primeiro),
//!   colegas na mesma partida (`same_team`) e, pela regra de não repetição,
//!   o campeão de outra partida da série e os de fora dela que a API passa
//!   (`Context::used`, `Context::used_by`). Se as regras não deixam nenhum,
//!   cedem: primeiro a de não repetição, depois a da partida (`GaveWay`),
//!   para o veto nunca travar. Só conta pick que é ou vai ser jogado
//!   (`Veto::counted_picks`, ver `draft.rs`).
//! - **Proteção do ban**: quem bane não pode tirar de si mesmo o que precisa
//!   para escolher todas as partidas quando os bans que o adversário ainda
//!   tem bastariam para esgotar (`Veto::options`).
//! - **O draft nunca fica pela metade**: o veto não começa com campeões de
//!   menos para os bans (`ChampionPoolExhausted`), e a partida 2 não começa
//!   com o draft da partida 1 por acabar (`DraftPending`). Quando o
//!   organizador lança a partida 1 antes, a API completa o que falta com o
//!   sorteio do tempo esgotado e só então começa a 2.
//! - Os passos da partida em andamento se desfazem do último para o
//!   primeiro (`Event::Undo`), o draft inteiro e o veto de mapas inclusive,
//!   enquanto a partida 1 não foi enviada; partida enviada, ou a série
//!   encerrada, não se desfaz.
//! - Os sorteios (mapas oferecidos, desempate, mapa aleatório e a escolha de
//!   quem deixou o tempo acabar) saem da seed gravada no começo: o mesmo
//!   histórico dá sempre o mesmo resultado (`draw.rs`).
//! - No `replay`, as regras de repetição e o pool de campeões não são
//!   conferidos de novo (valeram quando o passo foi gravado); a vez, o tipo
//!   do passo e o ban, sim. Na janela simultânea, o elenco também não: quem
//!   escolheu pode ter saído do time depois, e quem voltou pode ter
//!   escolhido de novo.

mod draft;
mod draw;
mod maps;
mod sequence;
mod team;
#[cfg(test)]
mod tests;
mod types;

use crate::core::MapId;
use crate::series::Side;

pub use draft::{GaveWay, PickOptions, Picker, Unavailable};
pub use sequence::{Action, DraftStep, draft_sequence, map_sequence};
pub use team::{Missing, Rosters};
pub use types::{
    Applied, Choice, Context, Event, MapMode, Target, Turn, VOTE_OPTIONS, VetoError, VetoRules,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Begin {
    seed: u64,
    first: Side,
}

/// O que foi feito com uma partida aberta: o veto de mapa dela e, na
/// partida 1, o draft; nos times, depois, as escolhas de quem entrou no
/// elenco.
#[derive(Clone, Debug, PartialEq, Eq)]
struct GameVeto {
    number: u8,
    /// Do evento `Game`: o mapa em que a partida anterior foi enviada.
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

/// O veto de uma série. Para reidratar, use `replay`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Veto {
    rules: VetoRules,
    best_of: u8,
    begin: Option<Begin>,
    games: Vec<GameVeto>,
    /// Eventos aplicados: separa os sorteios de tempo esgotado.
    events: u32,
    /// O que só a API sabe agora (`set_rosters`): quem escolhe campeão por
    /// lado nos times e o que já foi enviado. `None` até ser informado.
    rosters: Option<Rosters>,
}

/// Onde a partida em andamento está, sem olhar o elenco.
enum Phase {
    /// Fora da janela simultânea: a vez é esta.
    Turn(Turn),
    /// Nos times, a partida chegou à janela simultânea (o veto de mapa dela
    /// e, na partida 1, os bans acabaram): quem falta depende do elenco.
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

    /// Refaz o veto aplicando `events` em ordem, sem conferir as regras de
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

    /// Aplica um evento. Com `ctx`, o começo confere o pool de campeões
    /// contra os bans, um passo de campeão passa pelas regras de repetição e
    /// pelo pool (`options`, `team_options`), e a janela simultânea, pelo
    /// elenco. Erros: `AlreadyBegun`, `PoolTooSmall` e
    /// `ChampionPoolExhausted` (no começo), `NotBegun`, `GameOutOfOrder`,
    /// `DraftPending` (a partida 2 com o draft por acabar), `Finished`,
    /// `NotYourTurn`, `WrongAction`, `OptionInvalid`, `ChampionForbidden` (a
    /// regra de não repetição o tira), `ChampionTaken` (outro já o escolheu
    /// para a partida, sem espelho ou sem repetir no time), `AlreadyPicked`,
    /// `RostersUnknown` (janela simultânea com `ctx` sem o elenco informado)
    /// e `NothingToUndo`.
    pub fn apply(&mut self, event: Event, ctx: Option<&Context>) -> Result<(), VetoError> {
        match event {
            Event::Begin { seed, first } => self.begin(seed, first, ctx)?,
            Event::Game { game, previous } => {
                let next = self.games.len().saturating_add(1);
                if self.begin.is_none() {
                    return Err(VetoError::NotBegun);
                }
                if usize::from(game) != next || game > self.best_of {
                    let next = u8::try_from(next).unwrap_or(u8::MAX);
                    return Err(VetoError::GameOutOfOrder { next });
                }
                if self.draft_pending() {
                    return Err(VetoError::DraftPending);
                }
                self.games.push(GameVeto {
                    number: game,
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
                game,
                champion,
                gave_way,
            } => self.team_step(side, player, game, champion, gave_way, ctx)?,
            Event::Undo => {
                if self.undo_closed() {
                    return Err(VetoError::Finished);
                }
                let game = self.games.last_mut().ok_or(VetoError::NotBegun)?;
                game.steps.pop().ok_or(VetoError::NothingToUndo)?;
            }
        }
        self.events += 1;
        Ok(())
    }

    fn begin(&mut self, seed: u64, first: Side, ctx: Option<&Context>) -> Result<(), VetoError> {
        if self.begin.is_some() {
            return Err(VetoError::AlreadyBegun);
        }
        if self.best_of == 0 {
            return Err(VetoError::GameOutOfOrder { next: 1 });
        }
        if self.rules.map_mode != MapMode::Off {
            map_sequence(self.rules.pool.len(), self.best_of)?;
        }
        // Sobrar um campeão depois dos bans da série; o resto cede.
        let required = usize::from(self.rules.bans()) * 2 + 1;
        if let Some(ctx) = ctx
            && self.rules.champion_phase
            && ctx.champions.len() < required
        {
            return Err(VetoError::ChampionPoolExhausted {
                required,
                available: ctx.champions.len(),
            });
        }
        self.begin = Some(Begin { seed, first });
        self.games.push(GameVeto {
            number: 1,
            previous: None,
            steps: Vec::new(),
        });
        Ok(())
    }

    /// Com a partida 1 aberta e o draft de campeões ligado, ainda falta
    /// algum passo antes do fim dele (o mapa da partida 1 e o draft; na
    /// janela simultânea, com o elenco informado, alguém para alguma partida
    /// ainda aberta).
    fn draft_pending(&self) -> bool {
        if !self.rules.champion_phase || self.game() != Some(1) {
            return false;
        }
        match self.phase() {
            Phase::Turn(Turn::Act { .. }) => true,
            Phase::Turn(_) => false,
            Phase::TeamPick(_) => self.rosters.is_some() && !self.missing_now().is_empty(),
        }
    }

    /// O desfazer não vale mais: a partida em andamento foi enviada ou a
    /// série acabou (pelo que a API informou).
    fn undo_closed(&self) -> bool {
        self.game().is_some_and(|g| self.closed(g))
    }

    fn step(
        &mut self,
        side: Side,
        action: Action,
        choice: Choice,
        ctx: Option<&Context>,
    ) -> Result<(), VetoError> {
        if action == Action::TeamPick {
            // A janela simultânea é por jogador: `Event::TeamPick`.
            return Err(VetoError::WrongAction);
        }
        let (game, expected, target, sides, pick_for) = match self.turn() {
            Turn::NotBegun => return Err(VetoError::NotBegun),
            Turn::Done { .. } => return Err(VetoError::Finished),
            // A partida está na janela simultânea, que é por jogador.
            Turn::RostersUnknown { .. } => return Err(VetoError::WrongAction),
            Turn::Act {
                game,
                action,
                target,
                sides,
                pick_for,
            } => (game, action, target, sides, pick_for),
        };
        if !sides.contains(&side) {
            return Err(VetoError::NotYourTurn);
        }
        if action != expected {
            return Err(VetoError::WrongAction);
        }
        match (target, choice) {
            (Target::Map, Choice::Map(map)) => self.check_map(game, action, map)?,
            (Target::Champion, Choice::Champion(c)) => {
                self.check_draft(side, action, pick_for, c, ctx)?;
            }
            _ => return Err(VetoError::OptionInvalid),
        }
        let current = self.games.last_mut().ok_or(VetoError::NotBegun)?;
        current.steps.push(Applied {
            side,
            action,
            choice,
            player: None,
            game: pick_for,
            gave_way: false,
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

    /// A partida em andamento (a última que começou).
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

    /// A vez sem o elenco: na janela simultânea dos times, só a partida.
    fn phase(&self) -> Phase {
        let (Some(begin), Some(current)) = (self.begin, self.games.last()) else {
            return Phase::Turn(Turn::NotBegun);
        };
        let game = current.number;
        let (maps, champions) = current.split();
        if let Some(turn) = self.map_turn(begin, game, maps) {
            return Phase::Turn(turn);
        }
        if !self.rules.champion_phase {
            return Phase::Turn(Turn::Done { game });
        }
        if game == 1
            && let Some(step) = self.draft_sequence().get(champions.len())
        {
            return Phase::Turn(Turn::Act {
                game,
                action: step.action,
                target: Target::Champion,
                sides: vec![step.side],
                pick_for: step.game,
            });
        }
        if self.rules.teams() {
            Phase::TeamPick(game)
        } else {
            Phase::Turn(Turn::Done { game })
        }
    }

    /// Há passo para desfazer na partida em andamento.
    pub fn can_undo(&self) -> bool {
        !self.undo_closed() && self.games.last().is_some_and(|g| !g.steps.is_empty())
    }

    /// O que `side` pode escolher agora (vazio se não é a vez dele, e na
    /// janela simultânea dos times, que é por jogador: `team_options`): no
    /// veto de mapas, os que sobraram; na votação, os oferecidos; no draft,
    /// os bans e os picks do duelo (`draft.rs`).
    pub fn options(&self, side: Side, ctx: &Context) -> Vec<Choice> {
        let Turn::Act {
            game,
            action,
            target,
            sides,
            pick_for,
        } = self.turn()
        else {
            return Vec::new();
        };
        if !sides.contains(&side) {
            return Vec::new();
        }
        let champions = match (target, action) {
            (Target::Map, _) => return self.map_options(game, action),
            (Target::Champion, Action::Ban) => self.ban_options(side, ctx),
            (Target::Champion, Action::Pick) => {
                let game = pick_for.unwrap_or(game);
                self.pick_options(Picker { side, player: None }, game, ctx)
                    .champions
            }
            (Target::Champion, Action::Vote | Action::TeamPick) => Vec::new(),
        };
        champions.into_iter().map(Choice::Champion).collect()
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
            draw::Purpose::Timeout,
            game,
            self.events * 2 + index(side) as u32,
            options.len(),
        );
        options.get(i).copied()
    }
}
