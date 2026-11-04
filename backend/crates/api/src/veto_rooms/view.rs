//! A sala como cada link a vê: o veto pela mesma montagem da série de um torneio
//! (`matches::veto::build`), do ponto de vista do lugar; os links só para a
//! gestão.

use chrono::{DateTime, Utc};
use engine::core::PlayerId;

use super::{
    room::{Room, Seat},
    types::{VetoRoomDetail, VetoRoomLinks, VetoRoomRole, VetoRoomSeat, VetoRoomSeatLink},
};
use crate::matches::veto::{Board, Ctx, Shown, build};

impl Room {
    /// A sala como este link a vê.
    pub fn detail(&self, now: DateTime<Utc>) -> VetoRoomDetail {
        let ctx = Ctx::alone(&self.champions);
        let acting = self
            .seat()
            .filter(|s| s.name.is_some())
            .map(|s| (&ctx, s.side, PlayerId(s.id)));
        let board = Board {
            rules: &self.rules,
            step_seconds: self.step_seconds,
            team_pick_seconds: self.team_pick_seconds,
            maps: &self.maps,
            champions: &self.champions,
        };
        let names = |player: PlayerId| {
            self.seats
                .iter()
                .find(|s| s.id == player.0)
                .and_then(|s| s.name.clone())
                .unwrap_or_default()
        };
        let veto = build(
            &board,
            &Shown {
                log: Some(&self.log),
                state: &self.log.state,
                open: true,
                can_undo: false,
                reportable: false,
                acting,
                viewer: self.seat().map(|s| s.side),
                names: &names,
            },
        );
        VetoRoomDetail {
            now,
            expires_at: self.expires_at,
            role: self.role,
            seat: self.seat().map(public_seat),
            team_size: self.team_size,
            best_of: self.best_of,
            bans_per_series: self.bans_per_series,
            order: self.order,
            no_repeat: self.no_repeat,
            seats: self.seats.iter().map(public_seat).collect(),
            links: (self.role == VetoRoomRole::Manager).then(|| VetoRoomLinks {
                manage: self.manage_token.clone(),
                spectator: self.spectator_token.clone(),
                seats: self
                    .seats
                    .iter()
                    .map(|s| VetoRoomSeatLink {
                        seat: s.id,
                        token: s.token.clone(),
                    })
                    .collect(),
            }),
            veto,
        }
    }
}

fn public_seat(seat: &Seat) -> VetoRoomSeat {
    VetoRoomSeat {
        id: seat.id,
        side: seat.side,
        position: seat.position,
        name: seat.name.clone(),
    }
}
