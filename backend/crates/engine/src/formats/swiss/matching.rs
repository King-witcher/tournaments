//! Emparelhamento máximo num grafo qualquer (Edmonds, com contração de
//! flores), para o pareamento saber quantas revanches são inevitáveis.
//!
//! O grafo é a matriz `adj` (simétrica) restrita aos vértices `alive`. O
//! emparelhamento é `mate` (`NONE` para livre) e pode começar de qualquer
//! emparelhamento válido: `maximize` tenta aumentar a partir de cada vértice
//! livre uma vez só, o que basta (um vértice sem caminho de aumento continua
//! sem caminho depois de outros aumentos). Cada busca é O(n²), então partir
//! de um emparelhamento quase máximo custa pouco.

pub(super) const NONE: usize = usize::MAX;

/// Aumenta `mate` até ser máximo em `adj` restrito a `alive` e devolve o
/// tamanho (pares). `mate` só pode ligar vértices vivos por arestas de `adj`.
pub(super) fn maximize(adj: &[Vec<bool>], alive: &[bool], mate: &mut [usize]) -> usize {
    let n = adj.len();
    for root in 0..n {
        if alive[root]
            && mate[root] == NONE
            && let Some((end, parent)) = find_path(adj, alive, mate, root)
        {
            augment(mate, &parent, end);
        }
    }
    (0..n).filter(|&v| alive[v] && mate[v] != NONE).count() / 2
}

/// Inverte o caminho de aumento que termina no vértice livre `v`.
fn augment(mate: &mut [usize], parent: &[usize], mut v: usize) {
    while v != NONE {
        let pv = parent[v];
        let next = mate[pv];
        mate[v] = pv;
        mate[pv] = v;
        v = next;
    }
}

/// Busca em largura de um caminho de aumento a partir de `root` (livre).
/// Devolve o outro extremo (livre) e os pais da árvore, por onde o caminho
/// volta.
fn find_path(
    adj: &[Vec<bool>],
    alive: &[bool],
    mate: &[usize],
    root: usize,
) -> Option<(usize, Vec<usize>)> {
    let n = adj.len();
    let mut parent = vec![NONE; n];
    let mut base: Vec<usize> = (0..n).collect();
    let mut used = vec![false; n];
    let mut queue = vec![root];
    used[root] = true;
    let mut head = 0;
    while head < queue.len() {
        let v = queue[head];
        head += 1;
        for to in 0..n {
            if !alive[to] || !adj[v][to] || base[v] == base[to] || mate[v] == to {
                continue;
            }
            if to == root || (mate[to] != NONE && parent[mate[to]] != NONE) {
                // Ciclo ímpar: contrai a flor na base comum.
                let cur = common_base(&base, mate, &parent, v, to);
                let mut blossom = vec![false; n];
                mark_path(&base, mate, &mut parent, &mut blossom, v, cur, to);
                mark_path(&base, mate, &mut parent, &mut blossom, to, cur, v);
                for i in 0..n {
                    if blossom[base[i]] {
                        base[i] = cur;
                        if !used[i] {
                            used[i] = true;
                            queue.push(i);
                        }
                    }
                }
            } else if parent[to] == NONE {
                parent[to] = v;
                if mate[to] == NONE {
                    return Some((to, parent));
                }
                let next = mate[to];
                used[next] = true;
                queue.push(next);
            }
        }
    }
    None
}

/// A base mais próxima comum aos caminhos de `a` e `b` até a raiz.
fn common_base(base: &[usize], mate: &[usize], parent: &[usize], a: usize, b: usize) -> usize {
    let mut seen = vec![false; base.len()];
    let mut a = a;
    loop {
        a = base[a];
        seen[a] = true;
        if mate[a] == NONE {
            break;
        }
        a = parent[mate[a]];
    }
    let mut b = b;
    loop {
        b = base[b];
        if seen[b] {
            return b;
        }
        b = parent[mate[b]];
    }
}

/// Marca a flor de `v` até a base `b`, apontando os pais para dentro dela.
fn mark_path(
    base: &[usize],
    mate: &[usize],
    parent: &mut [usize],
    blossom: &mut [bool],
    mut v: usize,
    b: usize,
    mut child: usize,
) {
    while base[v] != b {
        blossom[base[v]] = true;
        blossom[base[mate[v]]] = true;
        parent[v] = child;
        child = mate[v];
        v = parent[mate[v]];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// Maior emparelhamento por força bruta.
    fn brute(adj: &[Vec<bool>], alive: &[bool]) -> usize {
        fn go(adj: &[Vec<bool>], free: &mut Vec<bool>) -> usize {
            let Some(v) = free.iter().position(|&f| f) else {
                return 0;
            };
            free[v] = false;
            let mut best = go(adj, free);
            for u in 0..adj.len() {
                if free[u] && adj[v][u] {
                    free[u] = false;
                    best = best.max(1 + go(adj, free));
                    free[u] = true;
                }
            }
            free[v] = true;
            best
        }
        go(adj, &mut alive.to_vec())
    }

    /// Grafo de `n` vértices com as arestas sorteadas em `edges`.
    fn graph(n: usize, edges: &[bool]) -> Vec<Vec<bool>> {
        let edge = |i: usize, j: usize| {
            let (a, b) = (i.min(j), i.max(j));
            i != j && edges[(a * n + b) % edges.len()]
        };
        (0..n)
            .map(|i| (0..n).map(|j| edge(i, j)).collect())
            .collect()
    }

    #[test]
    fn two_triangles_leave_one_pair_out() {
        let mut adj = vec![vec![false; 6]; 6];
        for (a, b) in [(0, 1), (1, 2), (0, 2), (3, 4), (4, 5), (3, 5)] {
            adj[a][b] = true;
            adj[b][a] = true;
        }
        let mut mate = vec![NONE; 6];
        assert_eq!(maximize(&adj, &[true; 6], &mut mate), 2);
    }

    #[test]
    fn warm_start_is_completed() {
        // Pentágono 0-1-2-3-4 com uma cauda 4-5, partindo de 0-1 e 2-3.
        let mut adj = vec![vec![false; 6]; 6];
        for (a, b) in [(0, 1), (1, 2), (2, 3), (3, 4), (4, 0), (4, 5)] {
            adj[a][b] = true;
            adj[b][a] = true;
        }
        let mut mate = vec![NONE; 6];
        mate[0] = 1;
        mate[1] = 0;
        mate[2] = 3;
        mate[3] = 2;
        assert_eq!(maximize(&adj, &[true; 6], &mut mate), 3);
    }

    proptest! {
        #[test]
        fn matches_brute_force(
            n in 1usize..=10,
            edges in proptest::collection::vec(any::<bool>(), 1..50),
            dead in proptest::collection::vec(any::<bool>(), 10),
            warm in any::<bool>(),
        ) {
            let adj = graph(n, &edges);
            let alive: Vec<bool> = (0..n).map(|i| !dead[i] || i % 3 == 0).collect();
            let mut mate = vec![NONE; n];
            if warm {
                // Começa de um emparelhamento guloso qualquer.
                for v in 0..n {
                    if !alive[v] || mate[v] != NONE {
                        continue;
                    }
                    if let Some(u) = (v + 1..n).find(|&u| alive[u] && mate[u] == NONE && adj[v][u]) {
                        mate[v] = u;
                        mate[u] = v;
                    }
                }
            }
            let size = maximize(&adj, &alive, &mut mate);
            prop_assert_eq!(size, brute(&adj, &alive));
            for v in 0..n {
                if mate[v] != NONE {
                    prop_assert!(alive[v] && alive[mate[v]] && adj[v][mate[v]]);
                    prop_assert_eq!(mate[mate[v]], v);
                }
            }
        }
    }
}
