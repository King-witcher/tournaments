//! Erros do `engine` como `ApiError`: um código estável por variante e os
//! números que a mensagem traduzida mostra.

use engine::core::EngineError;

use super::ApiError;

impl From<EngineError> for ApiError {
    fn from(err: EngineError) -> Self {
        use EngineError as E;
        let n = i64::from;
        match err {
            E::NotEnoughParticipants { min, got } => {
                Self::invalid("format.not_enough_participants")
                    .with_params([("min", min as i64), ("got", got as i64)])
            }
            E::TooManyParticipants { max, got } => Self::invalid("format.too_many_participants")
                .with_params([("max", max as i64), ("got", got as i64)]),
            E::DuplicateParticipant(_) => Self::invalid("format.duplicate_participant"),
            E::InvalidBestOf(best_of) => {
                Self::invalid("format.invalid_best_of").with_params([("best_of", n(best_of))])
            }
            E::NoStages => Self::invalid("format.no_stages"),
            E::UnknownMatch(_) => Self::not_found("match.not_found"),
            E::MatchNotReady(_) => Self::conflict("match.not_ready"),
            E::MatchAlreadyReported(_) => Self::conflict("match.already_reported"),
            E::NotInMatch { .. } => Self::invalid("match.not_in_match"),
            E::InvalidScore { best_of, .. } => {
                Self::invalid("match.invalid_score").with_params([("best_of", n(best_of))])
            }
            E::GameTied { game } => {
                Self::invalid("match.game_tied").with_params([("game", game as i64)])
            }
            E::MapRepeated(_) => Self::invalid("match.map_repeated"),
            E::FragsDisagree { game } => {
                Self::invalid("match.frags_disagree").with_params([("game", game as i64)])
            }
            E::SeriesDecided => Self::conflict("match.series_decided"),
            E::GameOutOfOrder { next } => {
                Self::conflict("match.game_out_of_order").with_params([("next", next as i64)])
            }
            E::NoGamesLeft { best_of } => {
                Self::conflict("match.no_games_left").with_params([("best_of", n(best_of))])
            }
            E::ChampionPoolExhausted {
                required,
                available,
            } => Self::invalid("champions.pool_exhausted").with_params([
                ("required", i64::from(required)),
                ("available", i64::from(available)),
            ]),
        }
    }
}
