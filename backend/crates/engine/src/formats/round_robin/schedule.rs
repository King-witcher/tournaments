//! As rodadas do round robin pelo método do círculo (tabela de Berger).

use crate::core::{BracketSide, Match, MatchId, ParticipantId};
use crate::formats::table::{pairing, rest};

/// Todas as partidas de `seeded` (na ordem dos seeds), em ordem de id.
///
/// Com N par, N − 1 rodadas; com N ímpar, entra um lugar vazio (N + 1) e há
/// N rodadas, cada uma com uma folga. Posições `0..m`; a última fica parada
/// e as outras giram: na rodada `r` (de 0), a parada enfrenta `r`, e para
/// `i` de 1 a `m/2 − 1`, `r + i` enfrenta `r − i` (módulo `m − 1`). Na
/// rodada 1, seed 1 contra N, 2 contra N − 1... (com N ímpar, o lugar vazio
/// é o da parada, então quem folga é o seed 1 na rodada 1, o 2 na 2...).
///
/// Em cada partida, o lado `a` é o seed melhor; na rodada, as partidas vão
/// na ordem do seed melhor de cada uma, e a folga é a última.
pub(super) fn build(seeded: &[ParticipantId], best_of: u8) -> Vec<Match> {
    let n = seeded.len();
    let m = n + n % 2;
    let turning = m - 1;
    let mut out = Vec::with_capacity(m / 2 * turning);
    for r in 0..turning {
        let mut pairs = vec![(r, turning)];
        for i in 1..m / 2 {
            pairs.push(((r + i) % turning, (r + turning - i) % turning));
        }
        let mut games: Vec<(usize, usize)> = Vec::new();
        let mut resting = None;
        for (x, y) in pairs {
            let (a, b) = (x.min(y), x.max(y));
            if b >= n {
                resting = Some(a);
            } else {
                games.push((a, b));
            }
        }
        games.sort_unstable();
        let id = |index: usize| MatchId {
            side: BracketSide::RoundRobin,
            round: r as u16 + 1,
            index: index as u16,
        };
        for (k, (a, b)) in games.iter().enumerate() {
            out.push(pairing(id(k + 1), seeded[*a], seeded[*b], best_of));
        }
        if let Some(p) = resting {
            out.push(rest(id(games.len() + 1), seeded[p], best_of));
        }
    }
    out
}
