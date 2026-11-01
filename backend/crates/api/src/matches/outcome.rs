//! O que chega para um mapa (mapa pelo slug e os frags de `a` e de `b`) → o
//! mapa que se grava, e o W.O. pelo lado. O vencedor sai do placar (quem fez
//! mais; `series::winner_from_frags`). As regras de série (mapa sem repetir,
//! próximo mapa) são do `engine`; aqui ficam os limites do site.

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
/// mapas ativos pelo slug). O mapa e o placar são sempre obrigatórios, para
/// o jogador e para o organizador (a tabela não aceita frags sem mapa), e o
/// vencedor é quem fez mais frags. Erros: 422 `match.map_required`,
/// `match.map_unknown`, `match.frags_invalid` `{min, max}` (fora da faixa) e
/// `match.game_tied` `{game}` (frags iguais).
pub fn game(
    number: i16,
    sides: (ParticipantId, ParticipantId),
    input: &GameInput,
    maps: &HashMap<String, MapId>,
) -> Result<NewGame, ApiError> {
    let slug = input
        .map
        .as_ref()
        .ok_or(ApiError::invalid("match.map_required"))?;
    let map = *maps
        .get(slug)
        .ok_or(ApiError::invalid("match.map_unknown"))?;
    let (a, b) = (input.a_frags, input.b_frags);
    let range = MIN_FRAGS..=MAX_FRAGS;
    if !range.contains(&a) || !range.contains(&b) {
        return Err(ApiError::invalid("match.frags_invalid")
            .with_params([("min", i64::from(MIN_FRAGS)), ("max", i64::from(MAX_FRAGS))]));
    }
    let game = usize::try_from(number).map_err(|_| ApiError::internal())?;
    let winner = Side::from(series::winner_from_frags(game, a, b)?);
    Ok(NewGame {
        number,
        sides,
        winner,
        map,
        frags: (a, b),
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

    fn input(map: Option<&str>, a_frags: i16, b_frags: i16) -> GameInput {
        GameInput {
            map: map.map(str::to_owned),
            a_frags,
            b_frags,
        }
    }

    fn code(r: Result<NewGame, ApiError>) -> &'static str {
        r.err().map_or("ok", |e| e.code)
    }

    #[test]
    fn the_score_decides_the_winner() {
        let g = game(2, SIDES, &input(Some("awoken"), 3, 9), &maps()).unwrap();
        assert_eq!(
            (g.winner_id(), g.winner, g.map, g.frags),
            (ParticipantId(20), Side::B, MapId(1), (3, 9))
        );
        let g = game(1, SIDES, &input(Some("awoken"), 9, 3), &maps()).unwrap();
        assert_eq!((g.winner_id(), g.winner), (ParticipantId(10), Side::A));
        // Suicídio e queda tiram frag: negativos valem, até o piso, e quem
        // fez mais vence mesmo abaixo de zero.
        let g = game(1, SIDES, &input(Some("awoken"), -999, -1), &maps()).unwrap();
        assert_eq!((g.winner_id(), g.frags), (ParticipantId(20), (-999, -1)));
        let g = game(1, SIDES, &input(Some("awoken"), 999, -999), &maps()).unwrap();
        assert_eq!(g.winner_id(), ParticipantId(10));
    }

    #[test]
    fn site_limits_and_engine_rules() {
        let m = maps();
        let check = |i: GameInput| code(game(1, SIDES, &i, &m));
        assert_eq!(check(input(None, 1, 0)), "match.map_required");
        assert_eq!(check(input(Some("q3dm17"), 1, 0)), "match.map_unknown");
        assert_eq!(check(input(Some("awoken"), 1000, 0)), "match.frags_invalid");
        assert_eq!(
            check(input(Some("awoken"), 0, -1000)),
            "match.frags_invalid"
        );
        let err = game(1, SIDES, &input(Some("awoken"), -1000, -1001), &m)
            .err()
            .unwrap();
        assert_eq!(err.params, [("min", -999), ("max", 999)]);
        // Empate não tem vencedor, nem com os dois no piso.
        let err = game(2, SIDES, &input(Some("awoken"), 10, 10), &m)
            .err()
            .unwrap();
        assert_eq!(err.code, "match.game_tied");
        assert_eq!(err.params, [("game", 2)]);
        assert_eq!(check(input(Some("awoken"), -999, -999)), "match.game_tied");
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
