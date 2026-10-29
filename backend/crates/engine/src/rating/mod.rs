//! Rating: o resultado de reprocessar, em ordem, os jogos e os ajustes
//! manuais de uma comunidade. Nada do estado é gravado como fonte: a API
//! refaz tudo do zero (`replay`) quando o histórico muda, então corrigir um
//! mapa antigo é reprocessar com o histórico corrigido.
//!
//! Um jogo é um mapa decidido (não a série; W.O. não entra). A ordem dos
//! eventos é de quem chama (a API ordena pelo horário em que o mapa
//! confirmou ou o ajuste foi feito, depois pelo id): `replay` só aplica, na
//! ordem dada, e a mesma lista dá sempre o mesmo resultado.
//!
//! `RatingSystem` é o sistema; `Elo` (em `elo`) é o primeiro. Os ratings são
//! inteiros e cada variação é arredondada no evento, para a soma das
//! variações exibidas bater com o rating exibido. O trait é por jogo, sem
//! tempo: um Glicko-2 de verdade agrupa os jogos por período e aumenta o
//! desvio com a inatividade, então ele vai pedir um evento de período (e
//! talvez um `Change` com o desvio) quando entrar.

mod elo;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use thiserror::Error;

pub use elo::{Elo, EloConfig, EloConfigError};

/// Uma pessoa no rating: o usuário (`users.id` na API). O rating é da pessoa
/// na comunidade, não da inscrição num torneio.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PlayerId(pub i64);

/// Com menos jogos que isto, o rating é provisório (ainda anda muito).
pub const PROVISIONAL_GAMES: u32 = 10;

/// Maior ajuste manual, para cima ou para baixo (o mesmo `check` está na
/// tabela da API).
pub const MAX_ADJUSTMENT: i32 = 1000;

/// Um sistema de rating.
pub trait RatingSystem {
    /// O que o sistema guarda de cada pessoa entre um evento e outro (Elo:
    /// só o rating; Glicko-2 guardaria também o desvio e a volatilidade).
    type State: Clone + std::fmt::Debug;

    /// O estado de quem ainda não tem nenhum evento.
    fn initial(&self) -> Self::State;

    /// O rating exibido.
    fn rating(&self, state: &Self::State) -> i32;

    /// Os estados do vencedor e do perdedor depois de um jogo entre os dois.
    /// `winner_games` e `loser_games`: quantos jogos cada um já tinha antes
    /// deste.
    fn game(
        &self,
        winner: &Self::State,
        winner_games: u32,
        loser: &Self::State,
        loser_games: u32,
    ) -> (Self::State, Self::State);

    /// O estado depois de um ajuste manual de `delta` pontos.
    fn adjust(&self, state: &Self::State, delta: i32) -> Self::State;
}

/// Um evento do histórico de rating.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// Um mapa decidido.
    Game { winner: PlayerId, loser: PlayerId },
    /// Ajuste manual da organização: soma `delta` ao rating e não conta como
    /// jogo (o K de quem foi ajustado não muda). `delta` diferente de zero,
    /// até `MAX_ADJUSTMENT` para cada lado.
    Adjustment { player: PlayerId, delta: i32 },
}

/// O que um evento fez com o rating de uma pessoa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Change {
    pub delta: i32,
    /// O rating depois do evento.
    pub after: i32,
}

/// O efeito de um evento, na mesma posição dele na lista.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    Game { winner: Change, loser: Change },
    Adjustment(Change),
}

/// Onde uma pessoa ficou depois de todos os eventos.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Standing {
    pub rating: i32,
    pub games: u32,
    pub wins: u32,
    pub losses: u32,
}

impl Standing {
    /// Menos de `PROVISIONAL_GAMES` jogos.
    pub const fn provisional(&self) -> bool {
        self.games < PROVISIONAL_GAMES
    }
}

/// O resultado de `replay`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Replay {
    /// Só quem apareceu em algum evento.
    pub standings: BTreeMap<PlayerId, Standing>,
    /// Um por evento, na mesma posição dele na lista: é pelo índice que quem
    /// chama liga o efeito ao evento.
    pub effects: Vec<Effect>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum RatingError {
    /// Um jogo de alguém contra si mesmo (a API nunca manda: os lados de um
    /// mapa são inscrições diferentes, de pessoas diferentes).
    #[error("o evento {index} é um jogo de {player:?} contra si mesmo")]
    SelfGame { index: usize, player: PlayerId },
    /// Ajuste zero ou além de `MAX_ADJUSTMENT`.
    #[error("o ajuste do evento {index} está fora de ±{max} (ou é zero)")]
    AdjustmentOutOfRange { index: usize, max: i32 },
}

/// Aplica `events` na ordem dada, começando todo mundo do estado inicial.
/// Um evento inválido recusa a lista inteira, sem resultado parcial.
pub fn replay<S: RatingSystem>(system: &S, events: &[Event]) -> Result<Replay, RatingError> {
    let mut people: BTreeMap<PlayerId, Person<S::State>> = BTreeMap::new();
    let mut effects = Vec::with_capacity(events.len());

    for (index, event) in events.iter().enumerate() {
        let effect = match *event {
            Event::Game { winner, loser } => {
                if winner == loser {
                    return Err(RatingError::SelfGame {
                        index,
                        player: winner,
                    });
                }
                let w = Person::of(&people, winner, system);
                let l = Person::of(&people, loser, system);
                let (w_after, l_after) = system.game(&w.state, w.games, &l.state, l.games);
                let effect = Effect::Game {
                    winner: change(system, &w.state, &w_after),
                    loser: change(system, &l.state, &l_after),
                };
                people.insert(winner, w.after_game(w_after, true));
                people.insert(loser, l.after_game(l_after, false));
                effect
            }
            Event::Adjustment { player, delta } => {
                if delta == 0 || delta.unsigned_abs() > MAX_ADJUSTMENT.unsigned_abs() {
                    return Err(RatingError::AdjustmentOutOfRange {
                        index,
                        max: MAX_ADJUSTMENT,
                    });
                }
                let mut p = Person::of(&people, player, system);
                let after = system.adjust(&p.state, delta);
                let effect = Effect::Adjustment(change(system, &p.state, &after));
                p.state = after;
                people.insert(player, p);
                effect
            }
        };
        effects.push(effect);
    }

    let standings = people
        .into_iter()
        .map(|(player, p)| {
            let standing = Standing {
                rating: system.rating(&p.state),
                games: p.games,
                wins: p.wins,
                losses: p.losses,
            };
            (player, standing)
        })
        .collect();
    Ok(Replay { standings, effects })
}

/// Uma pessoa durante o `replay`.
#[derive(Clone)]
struct Person<T> {
    state: T,
    games: u32,
    wins: u32,
    losses: u32,
}

impl<T: Clone> Person<T> {
    /// Como `player` está agora: o estado inicial se ainda não apareceu.
    fn of<S: RatingSystem<State = T>>(
        people: &BTreeMap<PlayerId, Self>,
        player: PlayerId,
        system: &S,
    ) -> Self {
        people.get(&player).cloned().unwrap_or_else(|| Self {
            state: system.initial(),
            games: 0,
            wins: 0,
            losses: 0,
        })
    }

    fn after_game(self, state: T, won: bool) -> Self {
        Self {
            state,
            games: self.games + 1,
            wins: self.wins + u32::from(won),
            losses: self.losses + u32::from(!won),
        }
    }
}

fn change<S: RatingSystem>(system: &S, before: &S::State, after: &S::State) -> Change {
    let after = system.rating(after);
    Change {
        delta: after.saturating_sub(system.rating(before)),
        after,
    }
}
