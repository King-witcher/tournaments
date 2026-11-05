//! O que os formatos de tabela (round robin e suíço) dividem: as partidas de
//! lados fixos (`League`) e a classificação com os desempates.
//!
//! Partidas: cada uma nasce com os dois participantes (`Ready`) ou com um só
//! (`Bye`: a folga do round robin, o bye do suíço). Um resultado só muda a
//! própria partida; nada depende de outra.
//!
//! Classificação:
//! - **saldo de séries** (primeiro critério do round robin, em qualquer melhor
//!   de N): vitórias menos derrotas, W.O. inclusive (vitória +1, empate 0,
//!   derrota −1). A folga não conta. Num round robin completo sem empates, a
//!   ordem é a mesma das vitórias: todos jogam N − 1 séries, então o saldo é
//!   `2 × vitórias − (N − 1)`; só a ordem durante o torneio muda (quem jogou
//!   mais não fica à frente só por isso);
//! - **pontos** (primeiro critério do suíço): séries vencidas (W.O. conta) e os
//!   byes (o bye vale vitória);
//! - **mapas**: os do placar da série. O W.O. vale o placar de quem vence
//!   sem perder mapa (2 x 0 na md2 e na md3, 3 x 0 na md4), para o vencedor não
//!   ficar atrás, no saldo, de quem jogou contra o mesmo adversário (assumido).
//!   A série empatada conta metade para cada lado. O bye não soma mapa;
//! - **frags**: os da série, só de série jogada, empatada inclusive, e com
//!   frags em todos os mapas (`StageReport::frags`); o W.O. não soma frag
//!   (assumido). Podem ser negativos, os pró também (suicídio e queda tiram
//!   frag): o saldo é pró menos contra, com sinal;
//! - **Buchholz** (só suíço): soma dos pontos atuais dos adversários das
//!   séries decididas, inclusive por W.O. e repetindo o adversário da
//!   revanche; o bye não soma nada, porque não há adversário (assumido);
//! - **ordem**: os critérios do formato, nesta ordem (`ROUND_ROBIN`,
//!   `SWISS`). O seed é o último e separa todos: a posição nunca empata;
//! - **confronto direto**: só quando todos os empatados já se enfrentaram
//!   (série decidida entre cada par, empatada inclusive; no suíço, é comum não
//!   terem jogado). É uma mini-tabela com as séries entre eles: no round robin,
//!   o saldo dentro do grupo (empate 0, então quem empatou entre si não se
//!   separa por ele); no suíço, as vitórias, porque com revanche o saldo daria
//!   outra ordem. Se separa o grupo, é refeita só entre os que continuam
//!   empatados; se não separa, vale o critério seguinte. Quando o saldo de
//!   mapas ou de frags separa um grupo, quem segue empatado volta ao confronto
//!   direto, só entre si, antes dos critérios seguintes;
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
use tiebreak::Meeting;

use crate::core::{Match, MatchState, Outcome, ParticipantId, SeriesFrags};
use crate::series::wins_needed;

/// Critério da ordem da tabela.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Criterion {
    SeriesDiff,
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
    /// A mini-tabela do confronto direto conta o saldo (round robin) ou só as
    /// vitórias (suíço). Sem revanche e sem empate, dão a mesma ordem; com
    /// revanche, não, e o suíço fica como sempre foi.
    pub mini_table_balance: bool,
}

/// Round robin: saldo de séries, confronto direto, saldo de mapas, saldo de
/// frags, seed.
pub(crate) const ROUND_ROBIN: Rules = Rules {
    bye_points: false,
    criteria: &[
        Criterion::SeriesDiff,
        Criterion::HeadToHead,
        Criterion::MapDiff,
        Criterion::FragDiff,
        Criterion::Seed,
    ],
    mini_table_balance: true,
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
    mini_table_balance: false,
};

/// Uma linha da tabela.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TableRow {
    pub participant: ParticipantId,
    /// De 1 a N, sem empate.
    pub position: u16,
    /// `wins + byes`. É o primeiro critério do suíço.
    pub points: u16,
    /// `wins - losses`. É o primeiro critério do round robin.
    pub series_diff: i32,
    /// Séries decididas, W.O. e empate inclusive; bye e folga não.
    pub played: u16,
    pub wins: u16,
    /// Séries empatadas: só nas séries pares.
    pub draws: u16,
    pub losses: u16,
    /// Byes do suíço, cada um valendo um ponto. No round robin, sempre 0 (a
    /// folga não vale ponto).
    pub byes: u16,
    pub maps_won: u16,
    pub maps_lost: u16,
    pub map_diff: i32,
    /// Frags das séries jogadas que vieram com frags; podem ser negativos.
    pub frags_for: i64,
    pub frags_against: i64,
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
    draws: u16,
    losses: u16,
    byes: u16,
    maps_won: u16,
    maps_lost: u16,
    frags_for: i64,
    frags_against: i64,
    /// Séries jogadas sem frags.
    frags_missing: u16,
    /// Adversários das séries decididas (com repetição, se houve revanche).
    opponents: Vec<usize>,
}

/// Os mapas de cada lado (`a`, `b`) de uma série decidida da melhor de
/// `best_of`. O W.O. vale o placar de quem vence sem perder mapa.
fn maps_of(a: ParticipantId, outcome: Outcome, best_of: u8) -> (u16, u16) {
    let (won, lost, a_won) = match outcome {
        Outcome::Played { winner, score } => (score.winner, score.loser, winner == a),
        Outcome::Walkover { winner } => (wins_needed(best_of), 0, winner == a),
        Outcome::Drawn { maps } => (maps, maps, true),
    };
    let (won, lost) = (u16::from(won), u16::from(lost));
    if a_won { (won, lost) } else { (lost, won) }
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
    // Cada série decidida, para o confronto direto.
    let mut meetings: Vec<Meeting> = Vec::new();
    for (m, f) in matches.iter().zip(frags) {
        match m.state {
            MatchState::Completed { a, b, outcome } => {
                let (Some(&ia), Some(&ib)) = (index.get(&a), index.get(&b)) else {
                    continue;
                };
                let winner = outcome.winner().map(|w| if w == a { ia } else { ib });
                match winner {
                    Some(w) => {
                        let l = if w == ia { ib } else { ia };
                        tally[w].wins += 1;
                        tally[l].losses += 1;
                    }
                    None => {
                        tally[ia].draws += 1;
                        tally[ib].draws += 1;
                    }
                }
                let (maps_a, maps_b) = maps_of(a, outcome, m.best_of);
                tally[ia].maps_won += maps_a;
                tally[ia].maps_lost += maps_b;
                tally[ib].maps_won += maps_b;
                tally[ib].maps_lost += maps_a;
                // O W.O. não foi jogado: nem soma frag nem conta como faltando.
                if !matches!(outcome, Outcome::Walkover { .. }) {
                    match f {
                        Some(f) => {
                            let (fa, fb) = (i64::from(f.a), i64::from(f.b));
                            tally[ia].frags_for += fa;
                            tally[ia].frags_against += fb;
                            tally[ib].frags_for += fb;
                            tally[ib].frags_against += fa;
                        }
                        None => {
                            tally[ia].frags_missing += 1;
                            tally[ib].frags_missing += 1;
                        }
                    }
                }
                tally[ia].opponents.push(ib);
                tally[ib].opponents.push(ia);
                meetings.push(Meeting {
                    a: ia,
                    b: ib,
                    winner,
                });
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
            Criterion::SeriesDiff => i64::from(t.wins) - i64::from(t.losses),
            Criterion::Points => i64::from(points[i]),
            Criterion::Buchholz => i64::from(buchholz[i]),
            Criterion::MapDiff => i64::from(t.maps_won) - i64::from(t.maps_lost),
            Criterion::FragDiff => t.frags_for - t.frags_against,
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

    tiebreak::order(participants.len(), rules, value, usable, &meetings)
        .into_iter()
        .enumerate()
        .map(|(k, (i, ahead_by))| {
            let t = &tally[i];
            TableRow {
                participant: participants[i],
                position: k as u16 + 1,
                points: points[i],
                series_diff: i32::from(t.wins) - i32::from(t.losses),
                played: t.wins + t.draws + t.losses,
                wins: t.wins,
                draws: t.draws,
                losses: t.losses,
                byes: t.byes,
                maps_won: t.maps_won,
                maps_lost: t.maps_lost,
                map_diff: i32::from(t.maps_won) - i32::from(t.maps_lost),
                frags_for: t.frags_for,
                frags_against: t.frags_against,
                frag_diff: t.frags_for - t.frags_against,
                frags_complete: complete(i),
                buchholz: with_buchholz.then_some(buchholz[i]),
                ahead_by,
            }
        })
        .collect()
}
