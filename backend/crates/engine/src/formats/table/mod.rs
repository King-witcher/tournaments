//! O que os formatos de tabela (round robin e suíço) dividem: as partidas de
//! lados fixos (`League`) e a classificação com os desempates.
//!
//! Partidas: cada uma nasce com os dois participantes (`Ready`) ou com um só
//! (`Bye`: a folga do round robin, o bye do suíço). Um resultado só muda a
//! própria partida; nada depende de outra.
//!
//! Classificação:
//! - **pontos**: séries vencidas (W.O. conta) e, no suíço, os byes (o bye
//!   vale vitória). A folga do round robin não vale ponto;
//! - **mapas**: os do placar da série. O W.O. vale o placar de quem vence
//!   sem perder mapa (2 x 0 na md3), para o vencedor não ficar atrás, no
//!   saldo, de quem jogou contra o mesmo adversário (assumido). O bye não
//!   soma mapa;
//! - **frags**: os da série, só de série jogada e com frags em todos os
//!   mapas (`StageReport::frags`); o W.O. não soma frag (assumido);
//! - **Buchholz** (só suíço): soma dos pontos atuais dos adversários das
//!   séries decididas, inclusive por W.O. e repetindo o adversário da
//!   revanche; o bye não soma nada, porque não há adversário (assumido);
//! - **ordem**: os critérios do formato, nesta ordem (`ROUND_ROBIN`,
//!   `SWISS`). O seed é o último e separa todos: a posição nunca empata;
//! - **confronto direto**: só quando todos os empatados já se enfrentaram
//!   (série decidida entre cada par; no suíço, é comum não terem jogado). É
//!   uma mini-tabela com as séries entre eles (as que cada um venceu contra
//!   os outros do grupo); se separa o grupo, é refeita só entre os que
//!   continuam empatados; se não separa, vale o critério seguinte;
//! - **saldo de frags**: só quando nenhum dos empatados tem série jogada sem
//!   frags; senão, vale o seed (comparar frags de séries diferentes daria
//!   vantagem a quem informa).
//!
//! A tabela em ordem é também a semente de uma fase seguinte (os N
//! primeiros para os playoffs).

mod league;
#[cfg(test)]
mod tests;
mod tiebreak;

pub(crate) use league::{League, pairing, rest};

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::core::{Match, MatchState, Outcome, ParticipantId, SeriesFrags};
use crate::series::wins_needed;

/// Critério da ordem da tabela.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Criterion {
    Points,
    Buchholz,
    HeadToHead,
    MapDiff,
    FragDiff,
    Seed,
}

/// Como um formato de tabela pontua e desempata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Rules {
    /// O bye vale vitória (suíço) ou é só folga (round robin).
    pub bye_points: bool,
    /// Critérios em ordem; o último é sempre o seed.
    pub criteria: &'static [Criterion],
}

/// Round robin: pontos, confronto direto, saldo de mapas, saldo de frags,
/// seed.
pub(crate) const ROUND_ROBIN: Rules = Rules {
    bye_points: false,
    criteria: &[
        Criterion::Points,
        Criterion::HeadToHead,
        Criterion::MapDiff,
        Criterion::FragDiff,
        Criterion::Seed,
    ],
};

/// Suíço: pontos, Buchholz, confronto direto, saldo de mapas, saldo de
/// frags, seed.
pub(crate) const SWISS: Rules = Rules {
    bye_points: true,
    criteria: &[
        Criterion::Points,
        Criterion::Buchholz,
        Criterion::HeadToHead,
        Criterion::MapDiff,
        Criterion::FragDiff,
        Criterion::Seed,
    ],
};

/// Uma linha da tabela.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TableRow {
    pub participant: ParticipantId,
    /// De 1 a N, sem empate.
    pub position: u16,
    /// `wins + byes`.
    pub points: u16,
    /// Séries decididas, W.O. inclusive; bye e folga não.
    pub played: u16,
    pub wins: u16,
    pub losses: u16,
    /// Byes do suíço, cada um valendo um ponto. No round robin, sempre 0 (a
    /// folga não vale ponto).
    pub byes: u16,
    pub maps_won: u16,
    pub maps_lost: u16,
    pub map_diff: i32,
    /// Frags das séries jogadas que vieram com frags.
    pub frags_for: u32,
    pub frags_against: u32,
    pub frag_diff: i64,
    /// Toda série jogada veio com frags: só assim o saldo de frags desempata.
    pub frags_complete: bool,
    /// Só no suíço.
    pub buchholz: Option<u32>,
    /// O critério que pôs esta linha à frente da seguinte; `None` na última.
    pub ahead_by: Option<Criterion>,
}

#[derive(Default)]
struct Tally {
    wins: u16,
    losses: u16,
    byes: u16,
    maps_won: u16,
    maps_lost: u16,
    frags_for: u32,
    frags_against: u32,
    /// Séries jogadas sem frags.
    frags_missing: u16,
    /// Adversários das séries decididas (com repetição, se houve revanche).
    opponents: Vec<usize>,
}

/// A tabela de `participants` (na ordem dos seeds) com as partidas e os
/// frags de cada uma (`frags[i]` é da partida `matches[i]`), já na ordem.
pub(crate) fn compute(
    rules: Rules,
    participants: &[ParticipantId],
    matches: &[Match],
    frags: &[Option<SeriesFrags>],
) -> Vec<TableRow> {
    let index: HashMap<ParticipantId, usize> = participants
        .iter()
        .enumerate()
        .map(|(i, p)| (*p, i))
        .collect();
    let mut tally: Vec<Tally> = participants.iter().map(|_| Tally::default()).collect();
    // (vencedor, perdedor) de cada série decidida, para o confronto direto.
    let mut results: Vec<(usize, usize)> = Vec::new();
    for (m, f) in matches.iter().zip(frags) {
        match m.state {
            MatchState::Completed { a, b, outcome } => {
                let (Some(&ia), Some(&ib)) = (index.get(&a), index.get(&b)) else {
                    continue;
                };
                let (w, l) = if outcome.winner() == a {
                    (ia, ib)
                } else {
                    (ib, ia)
                };
                tally[w].wins += 1;
                tally[l].losses += 1;
                // O W.O. vale o placar de quem vence sem perder mapa.
                let (score_w, score_l) = match outcome {
                    Outcome::Played { score, .. } => (score.winner, score.loser),
                    Outcome::Walkover { .. } => (wins_needed(m.best_of), 0),
                };
                let (score_w, score_l) = (u16::from(score_w), u16::from(score_l));
                tally[w].maps_won += score_w;
                tally[w].maps_lost += score_l;
                tally[l].maps_won += score_l;
                tally[l].maps_lost += score_w;
                if let Outcome::Played { .. } = outcome {
                    match f {
                        Some(f) => {
                            tally[w].frags_for += f.winner;
                            tally[w].frags_against += f.loser;
                            tally[l].frags_for += f.loser;
                            tally[l].frags_against += f.winner;
                        }
                        None => {
                            tally[w].frags_missing += 1;
                            tally[l].frags_missing += 1;
                        }
                    }
                }
                tally[ia].opponents.push(ib);
                tally[ib].opponents.push(ia);
                results.push((w, l));
            }
            MatchState::Bye {
                advanced: Some(p), ..
            } if rules.bye_points => {
                if let Some(&i) = index.get(&p) {
                    tally[i].byes += 1;
                }
            }
            _ => {}
        }
    }

    let points: Vec<u16> = tally.iter().map(|t| t.wins + t.byes).collect();
    let with_buchholz = rules.criteria.contains(&Criterion::Buchholz);
    let buchholz: Vec<u32> = tally
        .iter()
        .map(|t| t.opponents.iter().map(|&o| u32::from(points[o])).sum())
        .collect();
    let value = |c: Criterion, i: usize| -> i64 {
        let t = &tally[i];
        match c {
            Criterion::Points => i64::from(points[i]),
            Criterion::Buchholz => i64::from(buchholz[i]),
            Criterion::MapDiff => i64::from(t.maps_won) - i64::from(t.maps_lost),
            Criterion::FragDiff => i64::from(t.frags_for) - i64::from(t.frags_against),
            // O seed melhor (menor índice) fica à frente.
            Criterion::Seed => -(i as i64),
            // Calculado dentro de cada grupo de empatados (`tiebreak`).
            Criterion::HeadToHead => 0,
        }
    };

    let complete = |i: usize| tally[i].frags_missing == 0;
    // O saldo de frags só compara empatados que têm frags de todas as séries.
    let usable = |c: Criterion, group: &[usize]| match c {
        Criterion::FragDiff => group.iter().all(|&i| complete(i)),
        _ => true,
    };

    tiebreak::order(participants.len(), rules.criteria, value, usable, &results)
        .into_iter()
        .enumerate()
        .map(|(k, (i, ahead_by))| {
            let t = &tally[i];
            TableRow {
                participant: participants[i],
                position: k as u16 + 1,
                points: points[i],
                played: t.wins + t.losses,
                wins: t.wins,
                losses: t.losses,
                byes: t.byes,
                maps_won: t.maps_won,
                maps_lost: t.maps_lost,
                map_diff: i32::from(t.maps_won) - i32::from(t.maps_lost),
                frags_for: t.frags_for,
                frags_against: t.frags_against,
                frag_diff: i64::from(t.frags_for) - i64::from(t.frags_against),
                frags_complete: complete(i),
                buchholz: with_buchholz.then_some(buchholz[i]),
                ahead_by,
            }
        })
        .collect()
}
