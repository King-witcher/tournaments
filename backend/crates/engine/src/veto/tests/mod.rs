mod draft;
mod draft_props;
mod maps;
mod rosters;
mod team;
mod team_ban;
mod team_props;
mod undo;
mod vote;

use super::*;
use crate::core::{ChampionId, MapId, PlayerId};

pub const A: Side = Side::A;
pub const B: Side = Side::B;

pub fn pool(n: i64) -> Vec<MapId> {
    (1..=n).map(MapId).collect()
}

pub fn champions(n: i64) -> Vec<ChampionId> {
    (1..=n).map(|c| ChampionId(100 + c)).collect()
}

/// Duelo, com espelho e sem regra de não repetição.
pub fn rules(mode: MapMode, maps: i64, champion_phase: bool, bans: u8) -> VetoRules {
    VetoRules {
        map_mode: mode,
        pool: pool(maps),
        champion_phase,
        team_size: 1,
        champions: crate::champions::ChampionRules {
            bans_per_series: u32::from(bans),
            mirror: true,
            same_team: false,
            no_repeat: None,
        },
    }
}

/// As regras `r` com a regra de não repetição de `who` (o escopo fica com a
/// API: aqui, o da série).
pub fn with_rule(mut r: VetoRules, who: crate::champions::NoRepeatWho) -> VetoRules {
    r.champions.no_repeat = Some(crate::champions::NoRepeatRule {
        who,
        scope: crate::champions::NoRepeatScope::Series,
    });
    r
}

/// As regras `r` sem espelho.
pub fn no_mirror(mut r: VetoRules) -> VetoRules {
    r.champions.mirror = false;
    r
}

/// Duelo pela regra do jogador.
pub fn ruled(mode: MapMode, maps: i64, champion_phase: bool, bans: u8) -> VetoRules {
    with_rule(
        rules(mode, maps, champion_phase, bans),
        crate::champions::NoRepeatWho::Player,
    )
}

pub fn begun(rules: VetoRules, best_of: u8, first: Side) -> Veto {
    Veto::replay(rules, best_of, [Event::Begin { seed: 42, first }]).unwrap()
}

pub fn map(id: i64) -> Choice {
    Choice::Map(MapId(id))
}

pub fn champ(n: i64) -> Choice {
    Choice::Champion(ChampionId(100 + n))
}

/// O pool de campeões e o que cada lado usou fora da série.
pub fn ctx<'a>(champions: &'a [ChampionId], used: [&'a [ChampionId]; 2]) -> Context<'a> {
    Context {
        champions,
        used,
        used_by: &[],
    }
}

/// Só o pool de campeões.
pub fn free(champions: &[ChampionId]) -> Context<'_> {
    ctx(champions, [&[], &[]])
}

/// O passo esperado agora, para `side`.
pub fn step(veto: &mut Veto, side: Side, choice: Choice) -> Result<(), VetoError> {
    let action = match veto.turn() {
        Turn::Act { action, .. } => action,
        _ => Action::Pick,
    };
    veto.apply(
        Event::Step {
            side,
            action,
            choice,
        },
        None,
    )
}

pub fn step_in(
    veto: &mut Veto,
    side: Side,
    choice: Choice,
    ctx: &Context,
) -> Result<(), VetoError> {
    let action = match veto.turn() {
        Turn::Act { action, .. } => action,
        _ => Action::Pick,
    };
    veto.apply(
        Event::Step {
            side,
            action,
            choice,
        },
        Some(ctx),
    )
}

pub fn acting(veto: &Veto) -> (Action, Target, Vec<Side>) {
    match veto.turn() {
        Turn::Act {
            action,
            target,
            sides,
            ..
        } => (action, target, sides),
        other => panic!("esperava um passo, veio {other:?}"),
    }
}

/// O pick do duelo esperado agora: o lado e a partida a que se destina.
pub fn picking(veto: &Veto) -> (Side, u8) {
    match veto.turn() {
        Turn::Act {
            action: Action::Pick,
            target: Target::Champion,
            sides,
            pick_for: Some(game),
            ..
        } => (sides[0], game),
        other => panic!("esperava um pick, veio {other:?}"),
    }
}

/// A partida `game` começa (a anterior foi enviada).
pub fn next_game(v: &mut Veto, game: u8) {
    v.apply(
        Event::Game {
            game,
            previous: None,
        },
        None,
    )
    .unwrap();
}

pub fn ids(champions: &[i64]) -> Vec<ChampionId> {
    champions.iter().map(|c| ChampionId(100 + c)).collect()
}

// Nos times (TDM).

pub const P1: PlayerId = PlayerId(1);
pub const P2: PlayerId = PlayerId(2);
pub const P3: PlayerId = PlayerId(3);
pub const P4: PlayerId = PlayerId(4);
pub const P5: PlayerId = PlayerId(5);

/// Times de `size`, mapa combinado fora, com espelho.
pub fn team_rules(bans: u8, size: u8, same_team: bool) -> VetoRules {
    let mut r = VetoRules {
        team_size: size,
        ..rules(MapMode::Off, 0, true, bans)
    };
    r.champions.same_team = same_team;
    r
}

/// O elenco de agora, sem nada enviado.
pub fn open(a: &[PlayerId], b: &[PlayerId]) -> Rosters {
    Rosters {
        players: [a.to_vec(), b.to_vec()],
        lineups: Vec::new(),
        finished: false,
    }
}

/// O elenco de agora, com as `sent` primeiras partidas enviadas e jogadas
/// por `lineup`.
pub fn sent(a: &[PlayerId], b: &[PlayerId], sent: u8, lineup: [&[PlayerId]; 2]) -> Rosters {
    Rosters {
        lineups: (0..sent)
            .map(|_| [lineup[0].to_vec(), lineup[1].to_vec()])
            .collect(),
        ..open(a, b)
    }
}

/// 2v2, mapa combinado fora: A tem 1 e 2; B, 3 e 4.
pub fn two_v_two(rules: VetoRules, best_of: u8) -> Veto {
    let mut v = begun(rules, best_of, A);
    v.set_rosters(open(&[P1, P2], &[P3, P4])).unwrap();
    v
}

/// A escolha de `player` para a partida `game`, ao vivo.
pub fn pick(
    v: &mut Veto,
    side: Side,
    player: PlayerId,
    game: u8,
    c: i64,
    ctx: &Context,
) -> Result<(), VetoError> {
    v.apply(
        Event::TeamPick {
            side,
            player,
            game,
            champion: ChampionId(100 + c),
            gave_way: false,
        },
        Some(ctx),
    )
}

pub fn missing(v: &Veto) -> Vec<(Side, PlayerId, u8)> {
    v.missing()
        .into_iter()
        .map(|m| (m.side, m.player, m.game))
        .collect()
}

/// Os campeões escolhidos pelo lado `side` para a partida `game`, na ordem.
pub fn picks_of(v: &Veto, game: u8, side: Side) -> Vec<ChampionId> {
    v.team_picks(game)
        .iter()
        .filter(|a| a.side == side)
        .filter_map(|a| a.champion())
        .collect()
}

pub fn team_options(
    v: &Veto,
    side: Side,
    player: PlayerId,
    game: u8,
    ctx: &Context,
) -> Vec<ChampionId> {
    v.team_options(side, player, game, ctx).champions
}
