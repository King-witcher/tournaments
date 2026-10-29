//! O pareamento de uma rodada do suíço, determinístico (sem sorteio; o que
//! empata, a ordem da tabela resolve, e nela o último critério é o seed).
//!
//! Entrada: os ativos na ordem da tabela (índice 0 é o líder), com os
//! pontos, os byes e quem nunca se enfrentou.
//!
//! 1. **Bye**, com número ímpar de ativos: entre os que têm menos byes (quem
//!    não teve bye antes de quem já teve), o pior colocado. Se o bye dele
//!    obriga a mais revanches que o de outro desses candidatos, vale o
//!    próximo de baixo para cima que obrigue menos.
//! 2. **Revanches**: o mínimo possível no resto, normalmente zero. É o
//!    número de pares que faltam num emparelhamento máximo do grafo de quem
//!    nunca se enfrentou (`matching`). Só passa de zero quando não há
//!    pareamento sem revanche (poucos ativos, muitas rodadas, desistências):
//!    a regra cede só o necessário, em vez de travar a rodada.
//! 3. **Pares**: o melhor colocado ainda sem par fica com o primeiro
//!    adversário, na ordem de preferência, que deixa o resto parear com o
//!    mínimo de revanches. Preferência: primeiro os que ele nunca enfrentou,
//!    depois as revanches (a revanche inevitável cai nos de baixo da tabela);
//!    em cada parte, entre os de mesma pontuação ainda sem par (k, contando
//!    ele), a divisão em metades do sistema holandês: o de índice k/2, os de
//!    baixo dele, depois os de cima, do mais perto ao mais longe; depois os
//!    de pontuação menor, na ordem da tabela (o que sobra num grupo ímpar
//!    desce e enfrenta o melhor do grupo de baixo, mesmo que já tenha descido
//!    antes: simplificação do holandês, que evita repetir quem desce). Na
//!    rodada 1, todos com 0 pontos: seed 1 contra N/2 + 1, 2 contra
//!    N/2 + 2...

use super::matching::{NONE, maximize};

/// Pares (o primeiro é o melhor colocado, lado `a`) na ordem em que foram
/// formados, e o bye, em índices da entrada.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Pairing {
    pub pairs: Vec<(usize, usize)>,
    pub bye: Option<usize>,
}

/// `points`, `byes` e `fresh` (quem nunca se enfrentou, simétrica) dos
/// ativos na ordem da tabela.
pub(super) fn pair(points: &[u16], byes: &[u16], fresh: &[Vec<bool>]) -> Pairing {
    let k = points.len();
    let mut alive = vec![true; k];
    let mut mate = vec![NONE; k];
    let mut bye = None;
    let mut size = if k % 2 == 1 {
        let (c, m, size) = choose_bye(byes, fresh);
        alive[c] = false;
        mate = m;
        bye = Some(c);
        size
    } else {
        maximize(fresh, &alive, &mut mate)
    };
    let mut remaining = k - usize::from(bye.is_some());
    // Revanches que ainda vão ser necessárias.
    let mut rematches = remaining / 2 - size;

    let mut pairs = Vec::with_capacity(remaining / 2);
    while let Some(p) = alive.iter().position(|&a| a) {
        let mut chosen = None;
        // Os inéditos antes das revanches: a revanche inevitável fica para
        // os de baixo da tabela.
        let (new, old): (Vec<usize>, Vec<usize>) = preference(p, points, &alive)
            .into_iter()
            .partition(|&q| fresh[p][q]);
        for q in new.into_iter().chain(old) {
            let rematch = !fresh[p][q];
            if rematch && rematches == 0 {
                continue;
            }
            let allowed = rematches - usize::from(rematch);
            let mut next_alive = alive.clone();
            next_alive[p] = false;
            next_alive[q] = false;
            let mut next_mate = mate.clone();
            for v in [p, q] {
                if next_mate[v] != NONE {
                    let w = next_mate[v];
                    next_mate[w] = NONE;
                    next_mate[v] = NONE;
                }
            }
            let next_size = maximize(fresh, &next_alive, &mut next_mate);
            if (remaining - 2) / 2 - next_size == allowed {
                chosen = Some((q, next_alive, next_mate, next_size, allowed));
                break;
            }
        }
        // Sempre há escolha: o par de `p` num pareamento ótimo serve.
        let Some((q, next_alive, next_mate, next_size, allowed)) = chosen else {
            debug_assert!(false, "pareamento sem saída");
            break;
        };
        pairs.push((p, q));
        alive = next_alive;
        mate = next_mate;
        size = next_size;
        rematches = allowed;
        remaining -= 2;
    }
    debug_assert_eq!(remaining / 2, size + rematches);
    Pairing { pairs, bye }
}

/// O bye (passo 1): o candidato, o emparelhamento máximo do resto e o
/// tamanho dele.
fn choose_bye(byes: &[u16], fresh: &[Vec<bool>]) -> (usize, Vec<usize>, usize) {
    let k = byes.len();
    let fewest = byes.iter().copied().min().unwrap_or(0);
    let mut best: Option<(usize, Vec<usize>, usize)> = None;
    for c in (0..k).rev().filter(|&c| byes[c] == fewest) {
        let mut alive = vec![true; k];
        alive[c] = false;
        let mut mate = vec![NONE; k];
        let size = maximize(fresh, &alive, &mut mate);
        if best.as_ref().is_none_or(|b| size > b.2) {
            let perfect = size == (k - 1) / 2;
            best = Some((c, mate, size));
            if perfect {
                break;
            }
        }
    }
    best.expect("número ímpar de ativos tem candidato a bye")
}

/// Os adversários de `p` (o melhor colocado sem par) na ordem de
/// preferência (passo 3).
fn preference(p: usize, points: &[u16], alive: &[bool]) -> Vec<usize> {
    let open = |i: &usize| alive[*i] && *i != p;
    let same: Vec<usize> = (0..points.len())
        .filter(|i| alive[*i] && points[*i] == points[p])
        .collect();
    let half = same.len() / 2;
    let mut out: Vec<usize> = Vec::with_capacity(points.len());
    if same.len() >= 2 {
        out.extend(&same[half..]);
        out.extend(same[1..half].iter().rev());
    }
    out.retain(open);
    out.extend((0..points.len()).filter(|i| open(i) && points[*i] < points[p]));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_fresh(k: usize) -> Vec<Vec<bool>> {
        (0..k).map(|i| (0..k).map(|j| i != j).collect()).collect()
    }

    fn met(fresh: &mut [Vec<bool>], a: usize, b: usize) {
        fresh[a][b] = false;
        fresh[b][a] = false;
    }

    #[test]
    fn first_round_splits_in_halves() {
        let p = pair(&[0; 8], &[0; 8], &all_fresh(8));
        assert_eq!(p.pairs, vec![(0, 4), (1, 5), (2, 6), (3, 7)]);
        assert_eq!(p.bye, None);
    }

    #[test]
    fn odd_count_gives_the_bye_to_the_last() {
        let p = pair(&[0; 5], &[0; 5], &all_fresh(5));
        assert_eq!(p.bye, Some(4));
        assert_eq!(p.pairs, vec![(0, 2), (1, 3)]);
    }

    #[test]
    fn the_bye_skips_who_already_had_one() {
        let p = pair(&[2, 1, 1, 1, 0], &[0, 0, 0, 0, 1], &all_fresh(5));
        assert_eq!(p.bye, Some(3));
    }

    #[test]
    fn odd_score_group_floats_its_last_down() {
        // Três com 1 ponto, três com 0: 0-1 (k=3, índice 1), o 2 desce e
        // enfrenta o melhor de baixo (3), e 4-5.
        let p = pair(&[1, 1, 1, 0, 0, 0], &[0; 6], &all_fresh(6));
        assert_eq!(p.pairs, vec![(0, 1), (2, 3), (4, 5)]);
    }

    #[test]
    fn rematch_moves_to_the_next_preference() {
        let mut fresh = all_fresh(4);
        met(&mut fresh, 0, 2);
        let p = pair(&[1, 1, 1, 1], &[0; 4], &fresh);
        assert_eq!(p.pairs, vec![(0, 3), (1, 2)]);
    }

    #[test]
    fn looks_ahead_instead_of_getting_stuck() {
        // 0 prefere 2, mas 0-2 deixaria 1 e 3, que já se enfrentaram.
        let mut fresh = all_fresh(4);
        met(&mut fresh, 1, 3);
        let p = pair(&[0; 4], &[0; 4], &fresh);
        assert_eq!(p.pairs, vec![(0, 3), (1, 2)]);
    }

    #[test]
    fn rematches_only_when_unavoidable() {
        // Seis jogadores que já jogaram todos contra os de outro trio: sobram
        // dois triângulos, e um par precisa ser revanche.
        let mut fresh = all_fresh(6);
        for a in 0..3 {
            for b in 3..6 {
                met(&mut fresh, a, b);
            }
        }
        let p = pair(&[0; 6], &[0; 6], &fresh);
        let rematches = p.pairs.iter().filter(|(a, b)| !fresh[*a][*b]).count();
        assert_eq!(rematches, 1);
        assert_eq!(p.pairs.len(), 3);
    }

    #[test]
    fn the_bye_moves_up_to_avoid_a_rematch() {
        // Com o bye do último (4), o 0 só teria revanche (já enfrentou 1, 2
        // e 3); com o bye do 3, sobram 0-4 e 1-2.
        let mut fresh = all_fresh(5);
        met(&mut fresh, 0, 1);
        met(&mut fresh, 0, 2);
        met(&mut fresh, 0, 3);
        let p = pair(&[0; 5], &[0; 5], &fresh);
        assert_eq!(p.bye, Some(3));
        assert!(p.pairs.iter().all(|(a, b)| fresh[*a][*b]));
    }

    #[test]
    fn the_unavoidable_rematch_goes_down_the_table() {
        // Só 0, 1 e 3 não se enfrentaram entre si: uma revanche é certa. O
        // líder prefere o 2 (metade de baixo), mas é revanche; fica com o 3,
        // e a revanche é 1-2.
        let mut fresh = all_fresh(4);
        met(&mut fresh, 0, 2);
        met(&mut fresh, 1, 2);
        met(&mut fresh, 2, 3);
        let p = pair(&[0; 4], &[0; 4], &fresh);
        assert_eq!(p.pairs, vec![(0, 3), (1, 2)]);
    }
}
