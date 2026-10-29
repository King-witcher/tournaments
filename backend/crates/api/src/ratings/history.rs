//! O histórico que o rating reprocessa (mapas confirmados e ajustes, na
//! ordem) e o cache que sai do `engine` (o rating de cada um e o efeito de
//! cada evento), na forma das tabelas `ratings` e `rating_history`. Um
//! ranking só por comunidade, para todos os modos: cada mapa entra com a
//! escalação de cada lado (no duelo, um jogador de cada lado).

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use engine::rating::{Change, Effect, Event, PlayerId, Replay};

/// Um evento do histórico, com o que o liga ao banco.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceKind {
    /// Um mapa confirmado (`match_games.id`), com os usuários que jogaram
    /// por cada lado (`match_game_players`).
    Game {
        id: i64,
        winners: Vec<i64>,
        losers: Vec<i64>,
    },
    /// Um ajuste (`rating_adjustments.id`).
    Adjustment { id: i64, user: i64, delta: i32 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    pub kind: SourceKind,
    /// Quando o mapa confirmou ou o ajuste foi feito.
    pub at: DateTime<Utc>,
}

impl Source {
    fn event(&self) -> Event {
        let players = |users: &[i64]| users.iter().copied().map(PlayerId).collect();
        match &self.kind {
            SourceKind::Game {
                winners, losers, ..
            } => Event::Game {
                winners: players(winners),
                losers: players(losers),
            },
            &SourceKind::Adjustment { user, delta, .. } => Event::Adjustment {
                player: PlayerId(user),
                delta,
            },
        }
    }

    /// Horário; no mesmo instante, os mapas antes dos ajustes; depois o id.
    fn order_key(&self) -> (DateTime<Utc>, u8, i64) {
        match self.kind {
            SourceKind::Game { id, .. } => (self.at, 0, id),
            SourceKind::Adjustment { id, .. } => (self.at, 1, id),
        }
    }
}

/// Os eventos de `sources` na ordem do reprocessamento.
pub fn events(sources: &mut [Source]) -> Vec<Event> {
    sources.sort_by_key(Source::order_key);
    sources.iter().map(Source::event).collect()
}

/// Uma linha de `ratings`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RatingRow {
    pub user_id: i64,
    pub rating: i32,
    pub games: i32,
    pub wins: i32,
    pub losses: i32,
    pub last_game_at: Option<DateTime<Utc>>,
}

/// Uma linha de `rating_history`. O adversário de um mapa sai da escalação
/// dele (`match_game_players`), não daqui.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryRow {
    pub user_id: i64,
    pub seq: i32,
    pub game_id: Option<i64>,
    pub adjustment_id: Option<i64>,
    pub won: Option<bool>,
    pub delta: i32,
    pub rating_after: i32,
    pub at: DateTime<Utc>,
}

/// O cache de um tenant, em ordem (`ratings` por usuário; `rating_history`
/// por usuário e `seq`), para comparar com o gravado.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Cache {
    pub ratings: Vec<RatingRow>,
    pub history: Vec<HistoryRow>,
}

impl Cache {
    /// O cache do `replay` de `sources` (já ordenadas por `events`).
    pub fn build(sources: &[Source], replay: &Replay) -> Self {
        let mut history = Vec::with_capacity(sources.len() * 2);
        for ((source, effect), seq) in sources.iter().zip(&replay.effects).zip(1..) {
            let row = |user_id, change: Change| HistoryRow {
                user_id,
                seq,
                game_id: None,
                adjustment_id: None,
                won: None,
                delta: change.delta,
                rating_after: change.after,
                at: source.at,
            };
            match (&source.kind, effect) {
                (
                    SourceKind::Game {
                        id,
                        winners,
                        losers,
                    },
                    Effect::Game {
                        winners: w,
                        losers: l,
                    },
                ) => {
                    for (users, changes, won) in [(winners, w, true), (losers, l, false)] {
                        for (&user, &change) in users.iter().zip(changes) {
                            history.push(HistoryRow {
                                game_id: Some(*id),
                                won: Some(won),
                                ..row(user, change)
                            });
                        }
                    }
                }
                (&SourceKind::Adjustment { id, user, .. }, &Effect::Adjustment(change)) => {
                    history.push(HistoryRow {
                        adjustment_id: Some(id),
                        ..row(user, change)
                    });
                }
                _ => unreachable!("o efeito tem o tipo do evento na mesma posição"),
            }
        }
        history.sort_by_key(|h| (h.user_id, h.seq));
        let mut last_game = BTreeMap::new();
        for h in history.iter().filter(|h| h.game_id.is_some()) {
            last_game
                .entry(h.user_id)
                .and_modify(|at: &mut DateTime<Utc>| *at = (*at).max(h.at))
                .or_insert(h.at);
        }

        let ratings = replay
            .standings
            .iter()
            .map(|(player, s)| RatingRow {
                user_id: player.0,
                rating: s.rating,
                games: s.games as i32,
                wins: s.wins as i32,
                losses: s.losses as i32,
                last_game_at: last_game.get(&player.0).copied(),
            })
            .collect();
        Self { ratings, history }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use engine::rating::{Elo, EloConfig, replay};

    use super::*;

    fn at(minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 11, 20, minute, 0).unwrap()
    }

    fn game(id: i64, winner: i64, loser: i64, minute: u32) -> Source {
        Source {
            kind: SourceKind::Game {
                id,
                winners: vec![winner],
                losers: vec![loser],
            },
            at: at(minute),
        }
    }

    fn adjustment(id: i64, user: i64, delta: i32, minute: u32) -> Source {
        Source {
            kind: SourceKind::Adjustment { id, user, delta },
            at: at(minute),
        }
    }

    #[test]
    fn order_is_time_then_games_then_id() {
        let mut sources = [
            adjustment(1, 7, 10, 5),
            game(9, 7, 8, 5),
            game(3, 8, 7, 5),
            game(1, 7, 8, 6),
            adjustment(2, 8, -5, 1),
        ];
        events(&mut sources);
        let order: Vec<_> = sources.iter().map(|s| s.order_key()).collect();
        assert_eq!(
            order,
            [
                (at(1), 1, 2),
                (at(5), 0, 3),
                (at(5), 0, 9),
                (at(5), 1, 1),
                (at(6), 0, 1)
            ]
        );
    }

    #[test]
    fn cache_has_both_sides_of_each_game() {
        let mut sources = [game(40, 7, 8, 1), adjustment(5, 8, 20, 2)];
        let elo = Elo::new(EloConfig::default()).unwrap();
        let replay = replay(&elo, &events(&mut sources)).unwrap();
        let cache = Cache::build(&sources, &replay);
        assert_eq!(
            cache.ratings,
            [
                RatingRow {
                    user_id: 7,
                    rating: 1540,
                    games: 1,
                    wins: 1,
                    losses: 0,
                    last_game_at: Some(at(1)),
                },
                RatingRow {
                    user_id: 8,
                    rating: 1480,
                    games: 1,
                    wins: 0,
                    losses: 1,
                    last_game_at: Some(at(1)),
                },
            ]
        );
        let history: Vec<_> = cache
            .history
            .iter()
            .map(|h| (h.user_id, h.seq, h.game_id, h.adjustment_id, h.won, h.delta))
            .collect();
        assert_eq!(
            history,
            [
                (7, 1, Some(40), None, Some(true), 40),
                (8, 1, Some(40), None, Some(false), -40),
                (8, 2, None, Some(5), None, 20),
            ]
        );
    }

    #[test]
    fn cache_has_every_player_of_a_team_game() {
        let mut sources = [Source {
            kind: SourceKind::Game {
                id: 41,
                winners: vec![7, 9],
                losers: vec![8, 10],
            },
            at: at(1),
        }];
        let elo = Elo::new(EloConfig::default()).unwrap();
        let replay = replay(&elo, &events(&mut sources)).unwrap();
        let cache = Cache::build(&sources, &replay);
        let history: Vec<_> = cache
            .history
            .iter()
            .map(|h| (h.user_id, h.won, h.delta))
            .collect();
        assert_eq!(
            history,
            [
                (7, Some(true), 40),
                (8, Some(false), -40),
                (9, Some(true), 40),
                (10, Some(false), -40),
            ]
        );
        assert!(cache.ratings.iter().all(|r| r.games == 1));
    }

    #[test]
    fn only_adjusted_has_no_last_game() {
        let mut sources = [adjustment(1, 7, -30, 1)];
        let elo = Elo::new(EloConfig::default()).unwrap();
        let replay = replay(&elo, &events(&mut sources)).unwrap();
        let cache = Cache::build(&sources, &replay);
        assert_eq!(cache.ratings[0].games, 0);
        assert_eq!(cache.ratings[0].last_game_at, None);
        assert_eq!(cache.ratings[0].rating, 1470);
    }
}
