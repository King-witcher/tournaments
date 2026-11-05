//! Veto de mapas e draft de campeões de uma série, refeitos do histórico de
//! eventos como a API faz (`Veto::replay` e, depois, `set_rosters`).
//!
//! O "Pronto" de cada jogador não entra aqui: é da API (eventos `ready` e
//! `Readiness`); para o engine, o histórico começa em `Event::Begin`.

mod duel;
mod maps;
mod team;

use engine::core::{ChampionId, MapId, PlayerId};
use engine::series::Side;
use engine::veto::{Action, Applied, Choice, Event, Rosters, Target, Turn, Veto, VetoRules};

pub const A: Side = Side::A;
pub const B: Side = Side::B;

fn side_text(side: Side) -> &'static str {
    match side {
        Side::A => "A",
        Side::B => "B",
    }
}

fn action_text(action: Action) -> &'static str {
    match action {
        Action::Ban => "ban",
        Action::Pick => "pick",
        Action::Vote => "vote",
        Action::TeamPick => "team_pick",
    }
}

pub fn step(side: Side, action: Action, choice: Choice) -> Event {
    Event::Step {
        side,
        action,
        choice,
    }
}

pub fn ban_map(side: Side, map: i64) -> Event {
    step(side, Action::Ban, Choice::Map(MapId(map)))
}

pub fn pick_map(side: Side, map: i64) -> Event {
    step(side, Action::Pick, Choice::Map(MapId(map)))
}

pub fn vote_map(side: Side, map: i64) -> Event {
    step(side, Action::Vote, Choice::Map(MapId(map)))
}

pub fn ban_champion(side: Side, champion: i64) -> Event {
    step(side, Action::Ban, Choice::Champion(ChampionId(champion)))
}

pub fn pick_champion(side: Side, champion: i64) -> Event {
    step(side, Action::Pick, Choice::Champion(ChampionId(champion)))
}

/// A escolha de `player` (do lado `side`) para a partida `game`.
pub fn team_pick(side: Side, player: i64, game: u8, champion: i64) -> Event {
    Event::TeamPick {
        side,
        player: PlayerId(player),
        game,
        champion: ChampionId(champion),
        gave_way: false,
    }
}

/// A partida `game` começa; `previous` é o mapa em que a anterior foi enviada.
pub fn next_game(game: u8, previous: i64) -> Event {
    Event::Game {
        game,
        previous: Some(MapId(previous)),
    }
}

pub fn players(ids: &[i64]) -> Vec<PlayerId> {
    ids.iter().map(|n| PlayerId(*n)).collect()
}

/// Refaz o veto como a API: `rules` da linha do torneio, a melhor de N da
/// série e o histórico em ordem. Um histórico que não fecha derruba o teste.
pub fn replay(rules: &VetoRules, best_of: u8, events: &[Event]) -> Veto {
    Veto::replay(rules.clone(), best_of, events.iter().copied())
        .unwrap_or_else(|e| panic!("o histórico do veto deixou de refazer: {e:?}"))
}

/// O elenco de agora e o que já foi enviado, que a API informa depois do replay.
pub fn with_rosters(mut veto: Veto, rosters: Rosters) -> Veto {
    veto.set_rosters(rosters)
        .unwrap_or_else(|e| panic!("o elenco foi recusado: {e:?}"));
    veto
}

/// `sent` partidas enviadas, todas jogadas por `played_by` (lado A, lado B).
pub fn rosters(a: &[i64], b: &[i64], sent: usize, played_by: [&[i64]; 2]) -> Rosters {
    Rosters {
        players: [players(a), players(b)],
        lineups: vec![[players(played_by[0]), players(played_by[1])]; sent],
        finished: false,
    }
}

fn map_text(map: Option<MapId>) -> String {
    map.map_or("-".into(), |m| format!("M{}", m.0))
}

fn choice_text(choice: Choice) -> String {
    match choice {
        Choice::Map(m) => map_text(Some(m)),
        Choice::Champion(c) => format!("C{}", c.0),
    }
}

/// `A ban M3`, `B pick C2`; no pick por jogador, `A P1 C1`.
fn applied_text(a: &Applied) -> String {
    match a.player {
        Some(player) => format!(
            "{} P{} {}",
            side_text(a.side),
            player.0,
            choice_text(a.choice)
        ),
        None => format!(
            "{} {} {}",
            side_text(a.side),
            action_text(a.action),
            choice_text(a.choice)
        ),
    }
}

fn list(items: impl Iterator<Item = String>) -> String {
    let items: Vec<String> = items.collect();
    if items.is_empty() {
        "-".into()
    } else {
        items.join(", ")
    }
}

/// `g1 A ban map`, `g1 B team_pick champion for g2`, `done g3`.
pub fn turn_text(turn: &Turn) -> String {
    match turn {
        Turn::NotBegun => "not begun".into(),
        Turn::Done { game } => format!("done g{game}"),
        Turn::RostersUnknown { game } => format!("rosters unknown g{game}"),
        Turn::Act {
            game,
            action,
            target,
            sides,
            pick_for,
        } => {
            let sides: Vec<&str> = sides.iter().map(|s| side_text(*s)).collect();
            let target = match target {
                Target::Map => "map",
                Target::Champion => "champion",
            };
            let for_game = pick_for.map_or(String::new(), |g| format!(" for g{g}"));
            format!(
                "g{game} {} {} {target}{for_game}",
                sides.join("+"),
                action_text(*action)
            )
        }
    }
}

/// O veto de ponta a ponta: quem começou, os passos de mapa, o decisivo, o
/// mapa de cada partida, os bans, os picks que contam de cada partida, quem
/// falta, a vez e o número da vez.
pub fn veto_text(veto: &Veto) -> String {
    let games = 1..=veto.best_of();
    let maps = list(veto.map_steps().iter().map(applied_text));
    let game_lines: Vec<String> = games
        .clone()
        .map(|g| {
            let map = map_text(veto.map_of(g));
            match veto.picker(g) {
                Some(side) => format!("game {g}: {map} picked by {}", side_text(side)),
                None if veto.decider().is_some() && veto.map_of(g) == veto.decider() => {
                    format!("game {g}: {map} decider")
                }
                None => format!("game {g}: {map}"),
            }
        })
        .collect();
    let picks: Vec<String> = games
        .map(|g| {
            format!(
                "picks game {g}: {}",
                list(veto.counted_picks(g).iter().map(applied_text))
            )
        })
        .collect();
    let missing = list(
        veto.missing()
            .iter()
            .map(|m| format!("{} P{} g{}", side_text(m.side), m.player.0, m.game)),
    );
    format!(
        "first: {}\nmaps: {maps}\ndecider: {}\n{}\nbans: {}\n{}\nmissing: {missing}\nturn: {}\nturn opened: {}",
        veto.first().map_or("-", side_text),
        map_text(veto.decider()),
        game_lines.join("\n"),
        list(veto.bans().iter().map(applied_text)),
        picks.join("\n"),
        turn_text(&veto.turn()),
        veto.turn_opened(),
    )
}

/// Os mapas oferecidos, os votos e o mapa de cada partida (votação e sorteio):
/// `game 2: offered M1 M2 M5, votes M1/M5, map M5`.
pub fn draws_text(veto: &Veto) -> String {
    (1..=veto.best_of())
        .filter(|g| veto.game().is_some_and(|current| *g <= current))
        .map(|g| {
            let offered: Vec<String> = veto
                .offered(g)
                .into_iter()
                .map(|m| map_text(Some(m)))
                .collect();
            let [a, b] = veto.votes(g);
            format!(
                "game {g}: offered [{}], votes {}/{}, map {}",
                offered.join(" "),
                map_text(a),
                map_text(b),
                map_text(veto.map_of(g))
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
