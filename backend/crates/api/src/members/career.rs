//! A carreira e o confronto direto de um membro, montados do que
//! `db::record` lê das séries decididas e dos mapas confirmados: o
//! retrospecto em séries, a sequência atual, títulos e pódios (da colocação
//! de cada torneio, a mesma do perfil) e, por adversário, séries e mapas em
//! lados opostos.

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use super::types::{
    GameRecord, MemberSeries, MemberTournament, OpponentRecord, PersonLabel, SeriesOutcome,
    SeriesRecord, Streak, TournamentLabel,
};
use crate::{
    my_tournaments::{MyPlacement, Situation},
    shared::ApiError,
    teams::StoredTeam,
};

/// Uma série decidida do membro, do lado dele.
#[derive(Debug, Clone)]
pub struct Series {
    pub tournament: TournamentLabel,
    pub match_key: String,
    /// O time do outro lado.
    pub opponent_team: i64,
    pub outcome: SeriesOutcome,
    /// Mapas do lado do membro e do outro; `None` no W.O.
    pub wins: Option<i16>,
    pub losses: Option<i16>,
    pub at: DateTime<Utc>,
    /// Quem estava do outro lado: a escalação dos mapas confirmados da série
    /// ou, sem nenhum (W.O.), os jogadores do time adversário.
    pub opponents: Vec<i64>,
}

/// Os mapas confirmados do membro.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GameTotals {
    pub record: GameRecord,
    pub frags_for: i32,
    pub frags_against: i32,
}

/// Os mapas do membro contra uma pessoa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpponentGames {
    pub user_id: i64,
    pub record: GameRecord,
    pub last_at: DateTime<Utc>,
}

impl SeriesRecord {
    fn add(&mut self, outcome: SeriesOutcome) {
        match outcome {
            SeriesOutcome::Won => self.wins += 1,
            SeriesOutcome::Lost => self.losses += 1,
            SeriesOutcome::Draw => self.draws += 1,
        }
    }

    fn total(self) -> i32 {
        self.wins + self.losses + self.draws
    }
}

/// O retrospecto em séries.
pub fn record(series: &[Series]) -> SeriesRecord {
    let mut out = SeriesRecord::default();
    for s in series {
        out.add(s.outcome);
    }
    out
}

/// A sequência das séries mais recentes com o mesmo resultado (`series` da
/// mais recente); o empate não é sequência e a interrompe.
pub fn streak(series: &[Series]) -> Option<Streak> {
    let won = match series.first()?.outcome {
        SeriesOutcome::Won => true,
        SeriesOutcome::Lost => false,
        SeriesOutcome::Draw => return None,
    };
    let same = if won {
        SeriesOutcome::Won
    } else {
        SeriesOutcome::Lost
    };
    let count = series.iter().take_while(|s| s.outcome == same).count();
    Some(Streak {
        won,
        count: i32::try_from(count).unwrap_or(i32::MAX),
    })
}

/// Títulos, pódios e a melhor colocação, das colocações já definidas (o
/// empate a decidir ainda não é colocação).
#[derive(Debug, PartialEq, Eq)]
pub struct Placements {
    pub titles: i32,
    pub podiums: i32,
    pub best: Option<MyPlacement>,
}

/// `podium`: a faixa que começa até ela é pódio.
pub fn placements(entries: &[MemberTournament], podium: u16) -> Placements {
    let mut out = Placements {
        titles: 0,
        podiums: 0,
        best: None,
    };
    for e in entries {
        if e.situation == Situation::TiePending {
            continue;
        }
        let Some(p) = e.placement else { continue };
        if e.situation == Situation::Champion {
            out.titles += 1;
        }
        if p.rank <= podium {
            out.podiums += 1;
        }
        if out.best.is_none_or(|b| (p.rank, p.tied) < (b.rank, b.tied)) {
            out.best = Some(p);
        }
    }
    out
}

/// As séries como a API mostra, com o time adversário de `teams` (que tem
/// todos os de `series`).
pub fn with_opponents(
    series: &[Series],
    teams: &[StoredTeam],
) -> Result<Vec<MemberSeries>, ApiError> {
    series
        .iter()
        .map(|s| {
            let opponent = teams
                .iter()
                .find(|t| t.id == s.opponent_team)
                .ok_or_else(ApiError::internal)?;
            Ok(MemberSeries {
                tournament: s.tournament.clone(),
                match_key: s.match_key.clone(),
                opponent: opponent.view(),
                outcome: s.outcome,
                wins: s.wins,
                losses: s.losses,
                at: s.at,
            })
        })
        .collect()
}

/// O confronto com cada pessoa que esteve do outro lado: as séries (pelos
/// `opponents` de cada uma) e os mapas, juntos por pessoa, de quem tem mais
/// séries a quem tem menos (no empate, mais mapas, depois o apelido e o id).
/// Quem não está em `people` (não deveria acontecer) fica de fora.
pub fn opponents(
    series: &[Series],
    games: &[OpponentGames],
    people: Vec<PersonLabel>,
) -> Vec<OpponentRecord> {
    let mut by_person: HashMap<i64, (SeriesRecord, GameRecord, DateTime<Utc>)> = HashMap::new();
    for s in series {
        for &user in &s.opponents {
            let entry = by_person.entry(user).or_insert((
                SeriesRecord::default(),
                GameRecord::default(),
                s.at,
            ));
            entry.0.add(s.outcome);
            entry.2 = entry.2.max(s.at);
        }
    }
    for g in games {
        let entry = by_person.entry(g.user_id).or_insert((
            SeriesRecord::default(),
            GameRecord::default(),
            g.last_at,
        ));
        entry.1 = g.record;
        entry.2 = entry.2.max(g.last_at);
    }
    let mut out: Vec<OpponentRecord> = people
        .into_iter()
        .filter_map(|person| {
            let (series, games, last_at) = by_person.remove(&person.user_id)?;
            Some(OpponentRecord {
                person,
                series,
                games,
                last_at,
            })
        })
        .collect();
    out.sort_by(|a, b| {
        b.series
            .total()
            .cmp(&a.series.total())
            .then((b.games.wins + b.games.losses).cmp(&(a.games.wins + a.games.losses)))
            .then_with(|| {
                a.person
                    .nickname
                    .to_lowercase()
                    .cmp(&b.person.nickname.to_lowercase())
            })
            .then(a.person.user_id.cmp(&b.person.user_id))
    });
    out
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::shared::Locale;

    fn at(minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 16, 20, minute, 0).unwrap()
    }

    fn series(outcome: SeriesOutcome, minute: u32, opponents: &[i64]) -> Series {
        Series {
            tournament: TournamentLabel {
                slug: "copa".into(),
                locale: Locale::Pt,
                name: "Copa".into(),
                translations: vec![],
            },
            match_key: "wb-1-1".into(),
            opponent_team: 1,
            outcome,
            wins: None,
            losses: None,
            at: at(minute),
            opponents: opponents.to_vec(),
        }
    }

    fn person(user_id: i64, nickname: &str) -> PersonLabel {
        PersonLabel {
            user_id,
            nickname: nickname.into(),
            avatar_url: None,
            active: true,
        }
    }

    use SeriesOutcome::{Draw, Lost, Won};

    #[test]
    fn streak_counts_the_latest_equal_results() {
        assert_eq!(streak(&[]), None);
        let s = [
            series(Won, 3, &[]),
            series(Won, 2, &[]),
            series(Lost, 1, &[]),
        ];
        assert_eq!(
            streak(&s),
            Some(Streak {
                won: true,
                count: 2
            })
        );
        let s = [series(Lost, 3, &[]), series(Won, 2, &[])];
        assert_eq!(
            streak(&s),
            Some(Streak {
                won: false,
                count: 1
            })
        );
        // O empate mais recente não é sequência; mais atrás, a interrompe.
        assert_eq!(streak(&[series(Draw, 3, &[]), series(Won, 2, &[])]), None);
        let s = [
            series(Won, 3, &[]),
            series(Draw, 2, &[]),
            series(Won, 1, &[]),
        ];
        assert_eq!(streak(&s).unwrap().count, 1);
        assert_eq!(
            record(&s),
            SeriesRecord {
                wins: 2,
                losses: 0,
                draws: 1
            }
        );
    }

    #[test]
    fn opponents_join_series_and_maps_per_person() {
        // Contra 7: duas séries (TDM com 7 e 8, depois duelo com 7) e 3 mapas;
        // contra 8: uma série e 1 mapa; contra 9: só um mapa de uma série
        // ainda aberta.
        let s = [series(Won, 5, &[7]), series(Lost, 2, &[7, 8])];
        let games = [
            OpponentGames {
                user_id: 7,
                record: GameRecord { wins: 2, losses: 1 },
                last_at: at(4),
            },
            OpponentGames {
                user_id: 8,
                record: GameRecord { wins: 0, losses: 1 },
                last_at: at(1),
            },
            OpponentGames {
                user_id: 9,
                record: GameRecord { wins: 1, losses: 0 },
                last_at: at(9),
            },
        ];
        let people = vec![person(9, "Zé"), person(8, "bia"), person(7, "Ana")];
        let out = opponents(&s, &games, people);
        let rows: Vec<_> = out
            .iter()
            .map(|o| (o.person.user_id, o.series, o.games, o.last_at))
            .collect();
        assert_eq!(
            rows,
            [
                (
                    7,
                    SeriesRecord {
                        wins: 1,
                        losses: 1,
                        draws: 0
                    },
                    GameRecord { wins: 2, losses: 1 },
                    at(5)
                ),
                (
                    8,
                    SeriesRecord {
                        wins: 0,
                        losses: 1,
                        draws: 0
                    },
                    GameRecord { wins: 0, losses: 1 },
                    at(2)
                ),
                (
                    9,
                    SeriesRecord::default(),
                    GameRecord { wins: 1, losses: 0 },
                    at(9)
                ),
            ]
        );
    }

    #[test]
    fn opponents_tie_by_maps_then_nickname() {
        let games = [
            OpponentGames {
                user_id: 1,
                record: GameRecord { wins: 1, losses: 0 },
                last_at: at(1),
            },
            OpponentGames {
                user_id: 2,
                record: GameRecord { wins: 1, losses: 0 },
                last_at: at(1),
            },
            OpponentGames {
                user_id: 3,
                record: GameRecord { wins: 1, losses: 1 },
                last_at: at(1),
            },
        ];
        let people = vec![person(1, "bia"), person(2, "Ana"), person(3, "Zé")];
        let ids: Vec<_> = opponents(&[], &games, people)
            .iter()
            .map(|o| o.person.user_id)
            .collect();
        assert_eq!(ids, [3, 2, 1]);
    }
}
