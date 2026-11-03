//! Round robin: onde cada série pronta está, que sai dos mapas enviados e do
//! veto, sem estado próprio. Daí saem a série sugerida a cada time (a regra é
//! do `engine`, `round_robin::suggest`), a série em andamento, o adversário
//! ocupado em outra e quem recebe `match.ready` quando alguém termina uma
//! série. Só indica: nada aqui trava envio nem veto, porque uma série
//! contestada ou abandonada prenderia o time.

use std::collections::{BTreeMap, BTreeSet};

use engine::{
    core::{Match, MatchId, MatchState, ParticipantId},
    formats::{
        self as ef, StageBracket,
        round_robin::{self, SeriesProgress},
    },
    series,
};

use super::{games::series_games, result::Side, stage::Stage};

/// Faltando ao menos tantos mapas, a série ainda demora cerca de meia hora:
/// vale o adversário jogar outra em vez de esperar.
const LONG_GAMES: u8 = 2;

/// Quem recebe `match.ready`: a partida, os lados dela e os times avisados.
pub type Notices = Vec<(MatchId, (ParticipantId, ParticipantId), Vec<ParticipantId>)>;

/// A série sugerida a cada time e quem está jogando.
pub type Suggested = BTreeMap<ParticipantId, MatchId>;

fn league(stage: &Stage) -> bool {
    matches!(stage.format, ef::StageFormat::RoundRobin(_))
}

/// Onde a série pronta `m` está: nada enviado e o veto sem começar, aberta
/// (ou chamada, com um lado pronto para o veto); com mapa a jogar, em jogo; sem
/// mais mapa a jogar, só esperando confirmação ou a organização.
fn of(stage: &Stage, m: &Match) -> SeriesProgress {
    let games = stage.games_of(m.id);
    let veto = stage.veto_of(m.id);
    if games.is_empty() && !veto.is_some_and(|v| v.state.begun()) {
        return if veto.is_some_and(|v| v.ready().contains(&true)) {
            SeriesProgress::Called
        } else {
            SeriesProgress::Open
        };
    }
    let games = series_games(&games);
    if series::check_next(m.best_of, &games, None).is_err() {
        return SeriesProgress::Settling;
    }
    SeriesProgress::Playing {
        long: series::games_left(m.best_of, &games) >= LONG_GAMES,
    }
}

/// A série sugerida a cada time; vazio fora do round robin.
pub fn suggestions(stage: &Stage) -> Suggested {
    if !league(stage) {
        return Suggested::new();
    }
    round_robin::suggest(stage.bracket.matches(), |m| of(stage, m))
}

/// Os times com uma série em jogo; vazio fora do round robin.
pub fn playing(stage: &Stage) -> BTreeSet<ParticipantId> {
    if !league(stage) {
        return BTreeSet::new();
    }
    stage
        .bracket
        .matches()
        .iter()
        .filter_map(|m| match m.state {
            MatchState::Ready { a, b }
                if matches!(of(stage, m), SeriesProgress::Playing { .. }) =>
            {
                Some([a, b])
            }
            _ => None,
        })
        .flatten()
        .collect()
}

/// Round robin: a série `m` começou e ainda há mapa a jogar.
pub fn in_progress(stage: &Stage, m: &Match) -> bool {
    league(stage)
        && matches!(m.state, MatchState::Ready { .. })
        && matches!(of(stage, m), SeriesProgress::Playing { .. })
}

/// Round robin: quem vê joga `m` (do lado `mine`), que ainda não começou, e o
/// adversário está jogando outra (`playing`).
pub fn opponent_playing(
    stage: &Stage,
    m: &Match,
    mine: Option<Side>,
    playing: &BTreeSet<ParticipantId>,
) -> bool {
    let (Some(mine), MatchState::Ready { a, b }) = (mine, m.state) else {
        return false;
    };
    let opponent = match mine {
        Side::A => b,
        Side::B => a,
    };
    playing.contains(&opponent) && is_open(of(stage, m))
}

const fn is_open(progress: SeriesProgress) -> bool {
    matches!(progress, SeriesProgress::Open | SeriesProgress::Called)
}

/// Acrescenta o time `team` aos avisados da partida pronta `id`.
fn notify(notices: &mut Notices, bracket: &StageBracket, id: MatchId, team: ParticipantId) {
    if let Some(notice) = notices.iter_mut().find(|n| n.0 == id) {
        if !notice.2.contains(&team) {
            notice.2.push(team);
        }
    } else if let Some(MatchState::Ready { a, b }) = bracket.get(id).map(|m| m.state) {
        notices.push((id, (a, b), vec![team]));
    }
}

/// `match.ready` no início: cada time recebe só a série dele da vez, não as N
/// − 1 que já estão prontas.
pub fn first_notices(bracket: &StageBracket) -> Notices {
    let mut notices = Notices::new();
    let suggested = round_robin::suggest(bracket.matches(), |_| SeriesProgress::Open);
    for (team, id) in suggested {
        notify(&mut notices, bracket, id, team);
    }
    notices
}

/// Quem recebe `match.ready` depois de uma mudança.
///
/// Terminou uma série: o time recebe a próxima dele, a não ser que já esteja
/// em outra. Se o adversário dela está livre e a vez dele também é essa, recebe
/// junto, mesmo que já a conhecesse: era o outro lado que ele esperava. W.O.
/// numa série que ninguém começou e não era a da vez não muda nada para o time.
/// A série reaberta por uma correção avisa os dois lados.
///
/// `was` diz como a partida estava antes: o estado e se ninguém tinha mexido
/// nela.
pub fn notices(
    stage: &Stage,
    before: &Suggested,
    was: impl Fn(MatchId) -> Option<(MatchState, bool)>,
    suggested: &Suggested,
    playing: &BTreeSet<ParticipantId>,
) -> Notices {
    let bracket = &stage.bracket;
    let mut notices = Notices::new();
    let mut finished = BTreeSet::new();
    for m in bracket.matches() {
        let Some((state, untouched)) = was(m.id) else {
            continue;
        };
        match (state, m.state) {
            (MatchState::Ready { a, b }, MatchState::Completed { .. }) => {
                finished.extend(
                    [a, b]
                        .into_iter()
                        .filter(|side| !untouched || before.get(side) == Some(&m.id)),
                );
            }
            (MatchState::Completed { a, b, .. }, MatchState::Ready { .. }) => {
                notify(&mut notices, bracket, m.id, a);
                notify(&mut notices, bracket, m.id, b);
            }
            _ => {}
        }
    }
    for team in finished {
        let Some(&id) = suggested.get(&team) else {
            continue;
        };
        let Some(m) = bracket.get(id) else {
            continue;
        };
        let MatchState::Ready { a, b } = m.state else {
            continue;
        };
        if playing.contains(&team) || !is_open(of(stage, m)) {
            continue;
        }
        notify(&mut notices, bracket, id, team);
        let other = if a == team { b } else { a };
        if !playing.contains(&other) && suggested.get(&other) == Some(&id) {
            notify(&mut notices, bracket, id, other);
        }
    }
    notices
}
