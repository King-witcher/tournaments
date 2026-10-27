//! Montagem da topologia da chave: todas as partidas, com seus lados.

use super::{EliminationConfig, EliminationKind};
use crate::core::{BracketSide, Match, MatchId, MatchState, ParticipantId, Slot};
use crate::seeding::{bracket_order, bracket_size};
use crate::series::SeriesLength;

pub(super) fn mid(side: BracketSide, round: usize, index: usize) -> MatchId {
    MatchId {
        side,
        round: round as u16,
        index: index as u16,
    }
}

/// Posição (1-based) da chave de cima, na rodada `j + 1`, cujo perdedor
/// enfrenta o sobrevivente `i` na rodada `2j` da chave de baixo (`c` partidas).
///
/// Convenção: em `j` ímpar a ordem dos que descem é invertida (o perdedor da
/// última partida da rodada enfrenta o primeiro sobrevivente); em `j` par a
/// ordem é mantida. Com resultados de favorito em chaves cheias de 8, 16 e 32,
/// isso evita que quem desce reencontre, na primeira partida da chave de
/// baixo, alguém que já enfrentou. Exceção inevitável: a última descida (perdedor
/// da final de cima contra o campeão da chave de baixo), que com favoritos é
/// sempre o seed 2 contra o seed 3, que já se enfrentaram na chave de cima.
fn drop_position(j: usize, i: usize, c: usize) -> usize {
    if !j.is_multiple_of(2) { c + 1 - i } else { i }
}

pub(super) fn build_matches(config: EliminationConfig, seeded: &[ParticipantId]) -> Vec<Match> {
    let n = seeded.len();
    let size = bracket_size(n);
    let k = size.trailing_zeros() as usize;
    let double = matches!(config.kind, EliminationKind::Double { .. });
    let SeriesLength { default, finals } = config.series;
    let w = BracketSide::Winners;
    let l = BracketSide::Losers;
    let mut out = Vec::new();
    let push = |out: &mut Vec<Match>, id: MatchId, a: Slot, b: Slot, best_of: u8| {
        out.push(Match {
            id,
            a,
            b,
            best_of,
            state: MatchState::Pending,
        });
    };

    let order = bracket_order(size);
    let seat = |seed: usize| {
        if seed <= n {
            Slot::Participant(seeded[seed - 1])
        } else {
            Slot::Bye
        }
    };
    for i in 1..=size / 2 {
        let best_of = if !double && k == 1 { finals } else { default };
        push(
            &mut out,
            mid(w, 1, i),
            seat(order[2 * i - 2]),
            seat(order[2 * i - 1]),
            best_of,
        );
    }
    for r in 2..=k {
        let best_of = if !double && r == k { finals } else { default };
        for i in 1..=size >> r {
            push(
                &mut out,
                mid(w, r, i),
                Slot::WinnerOf(mid(w, r - 1, 2 * i - 1)),
                Slot::WinnerOf(mid(w, r - 1, 2 * i)),
                best_of,
            );
        }
    }

    if double {
        for rho in 1..=2 * (k - 1) {
            if rho == 1 {
                for i in 1..=size / 4 {
                    push(
                        &mut out,
                        mid(l, 1, i),
                        Slot::LoserOf(mid(w, 1, 2 * i - 1)),
                        Slot::LoserOf(mid(w, 1, 2 * i)),
                        default,
                    );
                }
            } else if rho.is_multiple_of(2) {
                let j = rho / 2;
                let c = size >> (j + 1);
                for i in 1..=c {
                    push(
                        &mut out,
                        mid(l, rho, i),
                        Slot::WinnerOf(mid(l, rho - 1, i)),
                        Slot::LoserOf(mid(w, j + 1, drop_position(j, i, c))),
                        default,
                    );
                }
            } else {
                let j = rho / 2;
                for i in 1..=size >> (j + 2) {
                    push(
                        &mut out,
                        mid(l, rho, i),
                        Slot::WinnerOf(mid(l, rho - 1, 2 * i - 1)),
                        Slot::WinnerOf(mid(l, rho - 1, 2 * i)),
                        default,
                    );
                }
            }
        }
        let gf = BracketSide::GrandFinal;
        let upper = Slot::WinnerOf(mid(w, k, 1));
        let lower = if k == 1 {
            Slot::LoserOf(mid(w, 1, 1))
        } else {
            Slot::WinnerOf(mid(l, 2 * (k - 1), 1))
        };
        push(&mut out, mid(gf, 1, 1), upper, lower, finals);
        if matches!(
            config.kind,
            EliminationKind::Double {
                grand_final_reset: true
            }
        ) {
            push(
                &mut out,
                mid(gf, 2, 1),
                Slot::LoserOf(mid(gf, 1, 1)),
                Slot::WinnerOf(mid(gf, 1, 1)),
                finals,
            );
        }
    }
    out.sort_by_key(|m| m.id);
    out
}
