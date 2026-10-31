//! Erros do `engine` (da chave e do veto) como `ApiError`: um código estável por variante e os
//! números que a mensagem traduzida mostra.

use engine::{core::EngineError, veto::VetoError};

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
            // Nos times: as vagas (jogadores) não formam dois times.
            E::NotEnoughPlayers { min, got } => Self::invalid("format.not_enough_players")
                .with_params([("min", min as i64), ("got", got as i64)]),
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
            E::InvalidTeamSize { max, got: _ } => Self::invalid("tournament.team_size_invalid")
                .with_params([("min", 1), ("max", i64::from(max))]),
            E::TooManyBans { max, got: _ } => {
                Self::invalid("tournament.bans_invalid").with_params([("max", i64::from(max))])
            }
            E::InvalidRounds { max, got } => Self::invalid("format.invalid_rounds")
                .with_params([("max", i64::from(max)), ("got", i64::from(got))]),
            // Histórico gravado que não fecha: erro nosso, não do usuário.
            E::InvalidPairing { .. } => Self::internal(),
        }
    }
}

impl From<VetoError> for ApiError {
    fn from(err: VetoError) -> Self {
        use VetoError as E;
        match err {
            E::NotBegun => Self::conflict("veto.not_begun"),
            E::AlreadyBegun => Self::conflict("veto.already_begun"),
            E::Finished => Self::conflict("veto.finished"),
            E::NotYourTurn => Self::conflict("veto.not_your_turn"),
            E::NothingToUndo => Self::conflict("veto.nothing_to_undo"),
            E::AlreadyPicked => Self::conflict("veto.already_picked"),
            E::OptionInvalid | E::WrongAction => Self::invalid("veto.option_invalid"),
            E::ChampionForbidden => Self::invalid("veto.champion_forbidden"),
            // Outro escolheu o mesmo campeão para a partida antes (sem espelho
            // ou sem repetir no time).
            E::ChampionTaken => Self::conflict("veto.champion_taken"),
            E::ChampionPoolExhausted {
                required,
                available,
            } => Self::invalid("champions.pool_exhausted").with_params([
                ("required", required as i64),
                ("available", available as i64),
            ]),
            E::PoolTooSmall { best_of, .. } => Self::invalid("tournament.map_pool_too_small")
                .with_params([("min", i64::from(best_of))]),
            E::PoolRepeated(_) => Self::invalid("tournament.map_pool_repeated"),
            // Só o histórico gravado e o elenco que a fase monta erram assim (ver
            // `MatchVeto::replay`); a API completa o draft antes de começar a
            // partida seguinte (`advance`).
            E::DraftPending
            | E::GameOutOfOrder { .. }
            | E::RostersUnknown
            | E::RosterRepeated(_)
            | E::RosterTooLarge { .. } => Self::internal(),
        }
    }
}
