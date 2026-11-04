//! Um passo do veto, antes de gravar: confere a vez que a tela mostrava, acha o
//! mapa ou o campeão pelo slug e passa pelo `engine`, que recusa o que não vale.
//! Serve à série de um torneio (`handlers::veto_step`) e à sala de veto
//! (`veto_rooms`): quem chama trava o escopo, confere quem pode agir, diz o lado
//! dele e grava. De quem é o campeão no pick por jogador sai de `target_player`.

use engine::{
    core::PlayerId,
    veto::{Action, Choice, Event, Picker, Turn, Veto},
};

use super::{
    Ctx,
    db::{Champion, NewVetoEvent},
    state::{Kind, VetoLog, ev_side},
    types::VetoChoice,
};
use crate::{matches::result::Side, shared::ApiError, tournaments::PoolMap};

const INVALID: ApiError = ApiError::invalid("veto.option_invalid");

/// A vez que a tela de quem age mostrava (`step`, se veio) ainda é a de
/// agora: um clique que chega depois de um sorteio ou do passo do outro não
/// cai no passo seguinte. 409 `veto.step_changed`.
pub fn seen(log: &VetoLog, step: Option<i64>) -> Result<(), ApiError> {
    match step {
        Some(step) if log.step() != Some(step) => Err(ApiError::conflict("veto.step_changed")),
        _ => Ok(()),
    }
}

/// O mapa (do pool) ou o campeão (ativo) do passo, pelo slug. 422
/// `veto.option_invalid` se não existe ou se não veio exatamente um.
pub fn choice(
    maps: &[PoolMap],
    champions: &[Champion],
    body: &VetoChoice,
) -> Result<Choice, ApiError> {
    match (&body.map, &body.champion) {
        (Some(slug), None) => maps
            .iter()
            .find(|m| &m.slug == slug)
            .map(|m| Choice::Map(m.id))
            .ok_or(INVALID),
        (None, Some(slug)) => champions
            .iter()
            .find(|c| c.active && &c.slug == slug)
            .map(|c| Choice::Champion(c.id))
            .ok_or(INVALID),
        _ => Err(INVALID),
    }
}

/// A vez que espera alguém.
#[derive(Clone, Copy)]
pub struct OpenTurn {
    game: u8,
    action: Action,
    pick_for: Option<u8>,
}

/// A vez de agora. 409 `veto.finished` (o veto da partida acabou) e
/// `veto.not_begun`.
pub fn open_turn(state: &Veto) -> Result<OpenTurn, ApiError> {
    match state.turn() {
        Turn::Act {
            game,
            action,
            pick_for,
            ..
        } => Ok(OpenTurn {
            game,
            action,
            pick_for,
        }),
        Turn::Done { .. } => Err(ApiError::conflict("veto.finished")),
        Turn::NotBegun => Err(ApiError::conflict("veto.not_begun")),
        // Quem refaz o veto sempre informa o elenco.
        Turn::RostersUnknown { .. } => Err(ApiError::internal()),
    }
}

/// De quem é o campeão no pick por jogador de `who`. Quem escolhe só por si
/// (`for_team` falso) escolhe o próprio: pedir outro é 403 `veto.not_your_pick`.
/// Quem escolhe pelo time diz o colega (`asked`; o `engine` recusa quem não é do
/// lado) ou fica com o próximo do lado sem escolha na partida `game`.
pub fn target_player(
    state: &Veto,
    turn: OpenTurn,
    (side, who): (Side, i64),
    for_team: bool,
    asked: Option<i64>,
    game: Option<i16>,
) -> Result<PlayerId, ApiError> {
    if turn.action != Action::TeamPick {
        return Ok(PlayerId(who));
    }
    if !for_team {
        return match asked {
            Some(other) if other != who => Err(ApiError::forbidden("veto.not_your_pick")),
            _ => Ok(PlayerId(who)),
        };
    }
    if let Some(asked) = asked {
        return Ok(PlayerId(asked));
    }
    let game = turn
        .pick_for
        .or_else(|| game.and_then(|g| u8::try_from(g).ok()));
    Ok(state
        .pickable()
        .into_iter()
        .find(|m| m.side == ev_side(side) && game.is_none_or(|g| m.game == g))
        .map_or(PlayerId(who), |m| m.player))
}

/// Quem faz o passo: o lado, quem clica (`actor`) e, no pick por jogador, de quem é
/// o campeão.
#[derive(Clone, Copy)]
pub struct Acting {
    pub side: Side,
    pub actor: i64,
    pub player: PlayerId,
}

/// O passo de `acting` na vez `turn`, conferido pelo `engine`, e o que gravar.
/// `target`: na janela simultânea, a partida para a qual ele escolhe (nos picks com
/// vez, é a que a vez diz, e outra é 409 `veto.step_changed`). Erros: os do
/// `engine` e 422 `veto.option_invalid` (mapa no pick por jogador, janela sem a
/// partida).
pub fn planned(
    state: &Veto,
    ctx: &Ctx,
    turn: OpenTurn,
    acting: Acting,
    choice: Choice,
    target: Option<i16>,
) -> Result<NewVetoEvent, ApiError> {
    let OpenTurn {
        game,
        action,
        pick_for,
    } = turn;
    let Acting {
        side,
        actor: who,
        player,
    } = acting;
    let (event, pick_game, gave_way) = match (action, choice) {
        (Action::TeamPick, Choice::Champion(champion)) => {
            let asked = target.and_then(|g| u8::try_from(g).ok());
            let target = match (pick_for, asked) {
                // A vez mudou sem evento (o elenco): a tela mostrava outra partida.
                (Some(turn), Some(asked)) if turn != asked => {
                    return Err(ApiError::conflict("veto.step_changed"));
                }
                (Some(turn), _) => turn,
                (None, Some(asked)) => asked,
                (None, None) => return Err(INVALID),
            };
            let picker = Picker {
                side: ev_side(side),
                player: Some(player),
            };
            let gave_way = state
                .pick_options(picker, target, &ctx.engine())
                .gave_way
                .is_some();
            let event = Event::TeamPick {
                side: ev_side(side),
                player,
                game: target,
                champion,
                gave_way,
            };
            (event, Some(target), gave_way)
        }
        (Action::TeamPick, Choice::Map(_)) => return Err(INVALID),
        (_, _) => {
            let event = Event::Step {
                side: ev_side(side),
                action,
                choice,
            };
            let pick_game = pick_for.filter(|_| matches!(choice, Choice::Champion(_)));
            (event, pick_game, false)
        }
    };
    state.clone().apply(event, Some(&ctx.engine()))?;
    let (map, champion) = match choice {
        Choice::Map(m) => (Some(m), None),
        Choice::Champion(c) => (None, Some(c)),
    };
    Ok(NewVetoEvent {
        map,
        champion,
        player: (action == Action::TeamPick).then_some(player.0),
        pick_game: pick_game.map(i16::from),
        gave_way,
        ..NewVetoEvent::new(
            Kind::of(action),
            Some(i16::from(game)),
            Some(side),
            Some(who),
        )
    })
}
