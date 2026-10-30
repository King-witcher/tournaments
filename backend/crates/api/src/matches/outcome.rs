//! O que chega para um mapa (vencedor pelo lado, mapa pelo slug, frags de
//! `a` e de `b`) → o mapa que se grava, e o W.O. pelo lado. As regras de
//! série (frags batendo com o vencedor, mapa sem repetir, próximo mapa) são
//! do `engine`; aqui ficam os limites do site.

use std::collections::HashMap;

use engine::{
    core::{MapId, Outcome, ParticipantId},
    series,
};

use super::{db::NewGame, result::Side, types::GameInput};
use crate::shared::ApiError;

/// Faixa dos frags de um lado num mapa (o mesmo check está na tabela). O
/// piso é negativo porque, no Quake Champions, suicídio e queda tiram frag.
pub const MIN_FRAGS: i16 = -999;
pub const MAX_FRAGS: i16 = 999;

/// O mapa `number` entre `sides`, com o mapa resolvido por `maps` (ids dos
/// mapas ativos pelo slug). `require_map`: o jogador sempre diz o mapa; o
/// organizador pode deixar sem (série antiga). Erros: 422
/// `match.map_required` (também frags sem mapa), `match.map_unknown`,
/// `match.frags_invalid` `{min, max}` (um lado só, ou fora da faixa) e os do
/// `engine` (`match.game_tied` `{game}`, `match.frags_disagree` `{game}`).
pub fn game(
    number: i16,
    sides: (ParticipantId, ParticipantId),
    input: &GameInput,
    maps: &HashMap<String, MapId>,
    require_map: bool,
) -> Result<NewGame, ApiError> {
    let map = match &input.map {
        Some(slug) => Some(
            *maps
                .get(slug)
                .ok_or(ApiError::invalid("match.map_unknown"))?,
        ),
        None => None,
    };
    let range = MIN_FRAGS..=MAX_FRAGS;
    let frags = match (input.a_frags, input.b_frags) {
        (None, None) => None,
        (Some(a), Some(b)) if range.contains(&a) && range.contains(&b) => Some((a, b)),
        _ => {
            return Err(ApiError::invalid("match.frags_invalid")
                .with_params([("min", i64::from(MIN_FRAGS)), ("max", i64::from(MAX_FRAGS))]));
        }
    };
    if map.is_none() && (require_map || frags.is_some()) {
        return Err(ApiError::invalid("match.map_required"));
    }
    let game = usize::try_from(number).map_err(|_| ApiError::internal())?;
    series::check_frags(game, input.winner.into(), frags)?;
    Ok(NewGame {
        number,
        sides,
        winner: input.winner.of(sides),
        map,
        frags,
    })
}

/// W.O. para o lado `winner`.
pub fn walkover(sides: (ParticipantId, ParticipantId), winner: Side) -> Outcome {
    Outcome::Walkover {
        winner: winner.of(sides),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIDES: (ParticipantId, ParticipantId) = (ParticipantId(10), ParticipantId(20));

    fn maps() -> HashMap<String, MapId> {
        [("awoken".to_owned(), MapId(1))].into_iter().collect()
    }

    fn input(winner: Side, map: Option<&str>, frags: (Option<i16>, Option<i16>)) -> GameInput {
        GameInput {
            winner,
            map: map.map(str::to_owned),
            a_frags: frags.0,
            b_frags: frags.1,
        }
    }

    fn code(r: Result<NewGame, ApiError>) -> &'static str {
        r.err().map_or("ok", |e| e.code)
    }

    #[test]
    fn a_map_with_or_without_frags() {
        let g = game(
            2,
            SIDES,
            &input(Side::B, Some("awoken"), (Some(3), Some(9))),
            &maps(),
            true,
        )
        .unwrap();
        assert_eq!(
            (g.winner, g.map, g.frags),
            (ParticipantId(20), Some(MapId(1)), Some((3, 9)))
        );
        let g = game(
            1,
            SIDES,
            &input(Side::A, Some("awoken"), (None, None)),
            &maps(),
            true,
        );
        assert_eq!(g.unwrap().frags, None);
        // Suicídio e queda tiram frag: negativos valem, até o piso.
        let g = game(
            1,
            SIDES,
            &input(Side::B, Some("awoken"), (Some(-999), Some(-1))),
            &maps(),
            true,
        );
        assert_eq!(g.unwrap().frags, Some((-999, -1)));
        // O organizador pode lançar sem o mapa (série antiga).
        let g = game(
            1,
            SIDES,
            &input(Side::A, None, (None, None)),
            &maps(),
            false,
        );
        assert_eq!(g.unwrap().map, None);
    }

    #[test]
    fn site_limits_and_engine_rules() {
        let m = maps();
        let check = |i: GameInput, require| code(game(1, SIDES, &i, &m, require));
        assert_eq!(
            check(input(Side::A, None, (None, None)), true),
            "match.map_required"
        );
        assert_eq!(
            check(input(Side::A, None, (Some(1), Some(0))), false),
            "match.map_required"
        );
        assert_eq!(
            check(input(Side::A, Some("q3dm17"), (None, None)), true),
            "match.map_unknown"
        );
        assert_eq!(
            check(input(Side::A, Some("awoken"), (Some(1000), Some(0))), true),
            "match.frags_invalid"
        );
        assert_eq!(
            check(input(Side::A, Some("awoken"), (Some(0), Some(-1000))), true),
            "match.frags_invalid"
        );
        let err = game(
            1,
            SIDES,
            &input(Side::A, Some("awoken"), (Some(-1000), Some(-1001))),
            &m,
            true,
        )
        .err()
        .unwrap();
        assert_eq!(err.params, [("min", -999), ("max", 999)]);
        assert_eq!(
            check(input(Side::A, Some("awoken"), (Some(10), None)), true),
            "match.frags_invalid"
        );
        assert_eq!(
            check(input(Side::A, Some("awoken"), (Some(10), Some(10))), true),
            "match.game_tied"
        );
        assert_eq!(
            check(input(Side::A, Some("awoken"), (Some(1), Some(10))), true),
            "match.frags_disagree"
        );
    }

    #[test]
    fn walkover_goes_to_the_side() {
        assert_eq!(
            walkover(SIDES, Side::B),
            Outcome::Walkover {
                winner: ParticipantId(20)
            }
        );
    }
}
