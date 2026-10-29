//! A situação de um inscrito num torneio: antes da chave, pelo status e pelo
//! check-in dele; com a chave, pelo lugar dele nela (`matches::Place`). Serve
//! a `/me/tournaments` e ao perfil do membro na comunidade (`members`).

use super::types::{MyPlacement, Situation};
use crate::{matches::Place, tournaments::TournamentStatus};

impl Situation {
    /// A situação antes da chave; `None` em andamento ou encerrado, em que
    /// ela sai do lugar na chave (`in_bracket`).
    pub const fn before_bracket(status: TournamentStatus, checked_in: bool) -> Option<Self> {
        match status {
            TournamentStatus::InProgress | TournamentStatus::Finished => None,
            TournamentStatus::CheckIn if checked_in => Some(Self::CheckedIn),
            TournamentStatus::CheckIn => Some(Self::CheckInPending),
            TournamentStatus::Draft | TournamentStatus::Registration => Some(Self::Registered),
        }
    }

    /// A situação pelo lugar na chave, com a colocação quando já se definiu.
    /// No TDM (`teams`), fora da chave é estar sem time (reserva ou
    /// substituído): iniciar exige todo time completo e com check-in.
    pub const fn in_bracket(place: Place, teams: bool) -> (Self, Option<MyPlacement>) {
        match place {
            Place::Missed if teams => (Self::Reserve, None),
            Place::Missed => (Self::MissedCheckIn, None),
            Place::Playing => (Self::Playing, None),
            Place::Withdrawn(None) => (Self::Withdrawn, None),
            Place::Withdrawn(Some((rank, tied))) => {
                (Self::Withdrawn, Some(MyPlacement { rank, tied }))
            }
            Place::Eliminated(rank, tied) => (Self::Eliminated, Some(MyPlacement { rank, tied })),
            Place::Champion => (Self::Champion, Some(MyPlacement { rank: 1, tied: 1 })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn before_the_bracket() {
        use TournamentStatus::{CheckIn, Draft, Finished, InProgress, Registration};
        for checked_in in [false, true] {
            assert_eq!(
                Situation::before_bracket(Draft, checked_in),
                Some(Situation::Registered)
            );
            assert_eq!(
                Situation::before_bracket(Registration, checked_in),
                Some(Situation::Registered)
            );
            assert_eq!(Situation::before_bracket(InProgress, checked_in), None);
            assert_eq!(Situation::before_bracket(Finished, checked_in), None);
        }
        assert_eq!(
            Situation::before_bracket(CheckIn, false),
            Some(Situation::CheckInPending)
        );
        assert_eq!(
            Situation::before_bracket(CheckIn, true),
            Some(Situation::CheckedIn)
        );
    }

    #[test]
    fn in_the_bracket() {
        let of = |place| {
            let (situation, placement) = Situation::in_bracket(place, false);
            (situation, placement.map(|p| (p.rank, p.tied)))
        };
        assert_eq!(of(Place::Missed), (Situation::MissedCheckIn, None));
        assert_eq!(
            Situation::in_bracket(Place::Missed, true).0,
            Situation::Reserve
        );
        assert_eq!(of(Place::Playing), (Situation::Playing, None));
        assert_eq!(of(Place::Withdrawn(None)), (Situation::Withdrawn, None));
        assert_eq!(
            of(Place::Withdrawn(Some((5, 4)))),
            (Situation::Withdrawn, Some((5, 4)))
        );
        assert_eq!(
            of(Place::Eliminated(3, 2)),
            (Situation::Eliminated, Some((3, 2)))
        );
        assert_eq!(of(Place::Champion), (Situation::Champion, Some((1, 1))));
    }
}
