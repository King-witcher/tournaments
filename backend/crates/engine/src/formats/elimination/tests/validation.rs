//! Limites e validação de entrada.

use super::*;

#[test]
fn participant_cap() {
    let max = MAX_PARTICIPANTS as u128;
    let full = EliminationBracket::new(double(true), seeds(max)).unwrap();
    assert!(full.worst_case_games() > 0);
    assert!(full.standings().is_empty());

    assert_eq!(
        EliminationBracket::new(double(true), seeds(max + 1)),
        Err(EngineError::TooManyParticipants {
            max: MAX_PARTICIPANTS,
            got: MAX_PARTICIPANTS + 1,
        })
    );
}
// ---- validação ----

#[test]
fn new_validates_input() {
    assert_eq!(
        EliminationBracket::new(single(3, 3), seeds(1)).unwrap_err(),
        EngineError::NotEnoughParticipants { min: 2, got: 1 }
    );
    assert_eq!(
        EliminationBracket::new(single(3, 3), vec![p(1), p(2), p(1)]).unwrap_err(),
        EngineError::DuplicateParticipant(p(1))
    );
    assert_eq!(
        EliminationBracket::new(single(2, 3), seeds(4)).unwrap_err(),
        EngineError::InvalidBestOf(2)
    );
    assert_eq!(
        EliminationBracket::new(single(3, 0), seeds(4)).unwrap_err(),
        EngineError::InvalidBestOf(0)
    );
}
