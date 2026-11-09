//! Torneio gravado → resposta da API, que `lifecycle` também devolve, e o
//! item de lista dele (`/tenants/{slug}/tournaments/{tournament}/mine`).

use super::{
    db::StoredTournament,
    types::{NameTranslation, Tournament, TournamentStage, TournamentSummary},
    veto::VetoSettings,
};
use crate::shared::public_url;

/// O mesmo item que `list` traria para este torneio.
impl From<&StoredTournament> for TournamentSummary {
    fn from(t: &StoredTournament) -> Self {
        Self {
            slug: t.slug.clone(),
            locale: t.locale,
            name: t.texts.name.clone(),
            mode: t.setup.mode,
            team_size: t.setup.team_size,
            status: t.status,
            admission: t.admission,
            max_participants: t.setup.max_participants,
            participants: t.participants,
            registered: t.registered,
            checked_in: t.checked_in,
            starts_at: t.schedule.starts_at,
            translations: t
                .translations
                .iter()
                .map(|tr| NameTranslation {
                    locale: tr.locale,
                    name: tr.name.clone(),
                })
                .collect(),
            created_at: t.created_at,
            cover_url: t.cover_key.as_deref().and_then(public_url),
        }
    }
}

impl From<StoredTournament> for Tournament {
    fn from(t: StoredTournament) -> Self {
        let stages = (1..)
            .zip(&t.setup.stages)
            .map(|(number, setup)| {
                let progress = t.stage(number).map(|(_, p)| p).unwrap_or_default();
                TournamentStage {
                    format: (*setup).into(),
                    number: u16::try_from(number).unwrap_or(1),
                    status: t.stage_status(number),
                    started_at: progress.started_at,
                    check_in_opened_at: progress.check_in_opened_at,
                }
            })
            .collect();
        Self {
            slug: t.slug,
            locale: t.locale,
            name: t.texts.name,
            description: t.texts.description,
            rules: t.texts.rules,
            mode: t.setup.mode,
            team_size: t.setup.team_size,
            status: t.status,
            admission: t.admission,
            max_participants: t.setup.max_participants,
            participants: t.participants,
            registered: t.registered,
            checked_in: t.checked_in,
            champion_rules: t.setup.champion_rules.into(),
            stages,
            starts_at: t.schedule.starts_at,
            check_in: t.schedule.check_in,
            contest_minutes: t.contest_minutes,
            counts_for_rating: t.counts_for_rating,
            veto: VetoSettings {
                map_mode: t.setup.veto.map_mode,
                order: t.setup.veto.order,
                map_pool: t.map_pool.into_iter().map(|m| m.slug).collect(),
                champion_phase: t.setup.veto.champion_phase,
                pick_order: t.setup.veto.pick_order,
                step_seconds: t.setup.veto.step_seconds,
                team_pick_seconds: t.setup.veto.team_pick_seconds,
            },
            team_rules: t.setup.team_rules,
            screenshots_required: t.screenshots_required,
            translations: t.translations,
            created_at: t.created_at,
            cover_url: t.cover_key.as_deref().and_then(public_url),
        }
    }
}
