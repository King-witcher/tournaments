mod champions;
mod maps;
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
        no_repeat: true,
        used,
    }
}

pub fn free(champions: &[ChampionId]) -> Context<'_> {
    Context {
        champions,
        no_repeat: false,
        used: [&[], &[]],
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
