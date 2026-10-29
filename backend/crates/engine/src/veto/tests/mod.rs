mod champions;
mod maps;
mod rosters;
mod team;
mod team_ban;
mod team_props;
mod undo;
mod vote;

use super::*;

pub const A: Side = Side::A;
pub const B: Side = Side::B;

pub fn pool(n: i64) -> Vec<MapId> {
    (1..=n).map(MapId).collect()
}

pub fn champions(n: i64) -> Vec<ChampionId> {
    (1..=n).map(|c| ChampionId(100 + c)).collect()
}

pub fn rules(mode: MapMode, maps: i64, champion_phase: bool, bans: u8) -> VetoRules {
    VetoRules {
        map_mode: mode,
        pool: pool(maps),
        champion_phase,
        champion_bans: bans,
        team_size: 1,
        team_duplicates: false,
    }
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

pub fn ctx<'a>(champions: &'a [ChampionId], used: [&'a [ChampionId]; 2]) -> Context<'a> {
    Context {
        champions,
        no_repeat: Some(crate::champions::NoRepeatWho::Player),
        used,
        used_by: &[],
    }
}

pub fn free(champions: &[ChampionId]) -> Context<'_> {
    Context {
        champions,
        no_repeat: None,
        used: [&[], &[]],
        used_by: &[],
    }
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

// Nos times (TDM).

pub const P1: PlayerId = PlayerId(1);
pub const P2: PlayerId = PlayerId(2);
pub const P3: PlayerId = PlayerId(3);
pub const P4: PlayerId = PlayerId(4);
pub const P5: PlayerId = PlayerId(5);

/// Times de `size`, mapa combinado fora.
pub fn team_rules(bans: u8, size: u8, duplicates: bool) -> VetoRules {
    VetoRules {
        team_size: size,
        team_duplicates: duplicates,
        ..rules(MapMode::Off, 0, true, bans)
    }
}

/// O elenco de agora, sem nada enviado.
pub fn open(a: &[PlayerId], b: &[PlayerId]) -> Rosters {
    Rosters {
        players: [a.to_vec(), b.to_vec()],
        sent: 0,
        finished: false,
    }
}

/// 2v2, mapa combinado fora, md3: A tem 1 e 2; B, 3 e 4.
pub fn two_v_two(bans: u8, duplicates: bool) -> Veto {
    let mut v = begun(team_rules(bans, 2, duplicates), 3, A);
    v.set_rosters(open(&[P1, P2], &[P3, P4])).unwrap();
    v
}

/// A escolha simultânea de `player`, ao vivo.
pub fn pick(
    v: &mut Veto,
    side: Side,
    player: PlayerId,
    c: i64,
    ctx: &Context,
) -> Result<(), VetoError> {
    v.apply(
        Event::TeamPick {
            side,
            player,
            champion: ChampionId(100 + c),
        },
        Some(ctx),
    )
}

/// O jogo `game` começa (A começa os campeões).
pub fn next_game(v: &mut Veto, game: u8) {
    v.apply(
        Event::Game {
            game,
            starter: A,
            previous: None,
        },
        None,
    )
    .unwrap();
}

/// Os campeões escolhidos pelo lado `side` no jogo `game`, na ordem.
pub fn picks_of(v: &Veto, game: u8, side: Side) -> Vec<ChampionId> {
    v.team_picks(game)
        .iter()
        .filter(|a| a.side == side)
        .filter_map(|a| match a.choice {
            Choice::Champion(c) => Some(c),
            Choice::Map(_) => None,
        })
        .collect()
}

pub fn ids(champions: &[i64]) -> Vec<ChampionId> {
    champions.iter().map(|c| ChampionId(100 + c)).collect()
}
