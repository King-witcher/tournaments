//! Qual série cada time deve jogar agora. Só indica: nada trava, e nenhum
//! resultado nem a tabela dependem disto.

use std::collections::{BTreeMap, BTreeSet};

use crate::core::{Match, MatchId, MatchState, ParticipantId};

/// Onde uma série pronta está. Quem chama (a API) sabe dos mapas enviados e do
/// veto; o engine não.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeriesProgress {
    /// Ninguém começou.
    Open,
    /// Um lado já pediu para começar (ficou pronto para o veto) e o outro
    /// ainda não.
    Called,
    /// Começou e ainda há mapa a jogar: os dois lados estão ocupados nela.
    /// `long`: ainda vai demorar (a API põe `true` quando faltam pelo menos 2
    /// mapas).
    Playing { long: bool },
    /// Começou, mas não há mais mapa a jogar (espera confirmação ou a
    /// organização): os lados estão livres.
    Settling,
}

/// Uma série pronta, com o progresso já consultado.
struct Series {
    id: MatchId,
    a: ParticipantId,
    b: ParticipantId,
    progress: SeriesProgress,
}

impl Series {
    fn other(&self, team: ParticipantId) -> ParticipantId {
        if self.a == team { self.b } else { self.a }
    }
}

struct Suggester {
    series: Vec<Series>,
    /// Para cada time, os índices (em `series`) das séries `Open` dele, em
    /// ordem.
    open_of: BTreeMap<ParticipantId, Vec<usize>>,
    /// Times em alguma série `Playing { long: true }`.
    long_playing: BTreeSet<ParticipantId>,
    /// Times que receberam sugestão nos passos 2, 3 ou 4.
    committed: BTreeSet<ParticipantId>,
    out: BTreeMap<ParticipantId, MatchId>,
}

impl Suggester {
    fn new(series: Vec<Series>) -> Self {
        let mut open_of: BTreeMap<ParticipantId, Vec<usize>> = BTreeMap::new();
        let mut long_playing = BTreeSet::new();
        for (i, s) in series.iter().enumerate() {
            match s.progress {
                SeriesProgress::Open => {
                    open_of.entry(s.a).or_default().push(i);
                    open_of.entry(s.b).or_default().push(i);
                }
                SeriesProgress::Playing { long: true } => {
                    long_playing.insert(s.a);
                    long_playing.insert(s.b);
                }
                _ => {}
            }
        }
        Self {
            series,
            open_of,
            long_playing,
            committed: BTreeSet::new(),
            out: BTreeMap::new(),
        }
    }

    fn free(&self, team: ParticipantId) -> bool {
        !self.out.contains_key(&team)
    }

    /// A primeira série `Open` do time, mesmo que já sugerida a outro.
    fn next_of(&self, team: ParticipantId) -> Option<usize> {
        self.open_of.get(&team)?.first().copied()
    }

    /// O adversário da vez ainda vai demorar: tem outra série antes desta, ou
    /// está tomado por algo longo.
    fn far(&self, team: ParticipantId) -> bool {
        let Some(n) = self.next_of(team) else {
            return false;
        };
        let opponent = self.series[n].other(team);
        self.next_of(opponent) != Some(n)
            || self.committed.contains(&opponent)
            || self.long_playing.contains(&opponent)
    }

    fn give(&mut self, index: usize, committed: bool) {
        let Series { id, a, b, .. } = self.series[index];
        for team in [a, b] {
            if self.free(team) {
                self.out.insert(team, id);
                if committed {
                    self.committed.insert(team);
                }
            }
        }
    }

    fn indices(&self, progress: impl Fn(SeriesProgress) -> bool) -> Vec<usize> {
        (0..self.series.len())
            .filter(|&i| progress(self.series[i].progress))
            .collect()
    }

    fn run(mut self) -> BTreeMap<ParticipantId, MatchId> {
        // 1. Em jogo: os dois lados estão ocupados nela.
        for i in self.indices(|p| matches!(p, SeriesProgress::Playing { .. })) {
            self.give(i, false);
        }
        // 2. Chamada: quem ficou pronto se comprometeu.
        for i in self.indices(|p| p == SeriesProgress::Called) {
            self.give(i, true);
        }
        let open = self.indices(|p| p == SeriesProgress::Open);
        // 3. Ordem das rodadas: a primeira série aberta dos dois.
        for &i in &open {
            let Series { a, b, .. } = self.series[i];
            if self.free(a)
                && self.free(b)
                && self.next_of(a) == Some(i)
                && self.next_of(b) == Some(i)
            {
                self.give(i, true);
            }
        }
        // 4. Salto: os dois adversários da vez ainda vão demorar.
        for &i in &open {
            let Series { a, b, .. } = self.series[i];
            if self.free(a) && self.free(b) && self.far(a) && self.far(b) {
                self.give(i, true);
            }
        }
        // 5. O que sobrou: a primeira aberta (o adversário pode estar
        // ocupado); sem nenhuma, a primeira em fechamento.
        for i in 0..self.series.len() {
            let Series { a, b, progress, .. } = self.series[i];
            for team in [a, b] {
                if !self.free(team) {
                    continue;
                }
                let first_open = self.next_of(team);
                let settling = first_open.is_none() && progress == SeriesProgress::Settling;
                if first_open == Some(i) || settling {
                    self.out.insert(team, self.series[i].id);
                }
            }
        }
        self.out
    }
}

/// A série que cada time deve jogar agora. `matches` na ordem das rodadas
/// (`RoundRobinBracket::matches`); `progress` só é chamado com partidas
/// `Ready`. Time sem nenhuma série pronta fica de fora.
///
/// A ordem das rodadas é o padrão; só se sai dela quando o adversário da vez
/// ainda vai demorar. Em ordem de prioridade (quem já tem sugestão não recebe
/// outra): a série em jogo; a chamada por um lado; a primeira série aberta
/// dos dois; o salto para uma série aberta entre dois times cujos adversários
/// da vez vão demorar; e, para o que sobrou, a primeira série aberta (ou, sem
/// nenhuma, a primeira em fechamento).
pub fn suggest(
    matches: &[Match],
    progress: impl Fn(&Match) -> SeriesProgress,
) -> BTreeMap<ParticipantId, MatchId> {
    let series = matches
        .iter()
        .filter_map(|m| match m.state {
            MatchState::Ready { a, b } => Some(Series {
                id: m.id,
                a,
                b,
                progress: progress(m),
            }),
            _ => None,
        })
        .collect();
    Suggester::new(series).run()
}
