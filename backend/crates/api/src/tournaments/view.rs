//! Torneio gravado → resposta da API, que `lifecycle` também devolve.

use super::{db::StoredTournament, types::Tournament, veto::VetoSettings};

impl From<StoredTournament> for Tournament {
    fn from(t: StoredTournament) -> Self {
        Self {
            slug: t.slug,
            locale: t.locale,
            name: t.texts.name,
            description: t.texts.description,
            rules: t.texts.rules,
            mode: t.mode,
            status: t.status,
            visibility: t.visibility,
            max_participants: t.setup.max_participants,
            participants: t.participants,
            registered: t.registered,
            checked_in: t.checked_in,
            champion_rules: t.setup.champion_rules.into(),
            stages: t.setup.stages.into_iter().map(Into::into).collect(),
            starts_at: t.schedule.starts_at,
            check_in_minutes: t.schedule.check_in_minutes,
            check_in_opens_at: t.schedule.check_in_opens_at(),
            contest_minutes: t.contest_minutes,
            counts_for_rating: t.counts_for_rating,
            veto: VetoSettings {
                map_mode: t.setup.veto.map_mode,
                map_pool: t.map_pool.into_iter().map(|m| m.slug).collect(),
                champion_phase: t.setup.veto.champion_phase,
                step_seconds: t.setup.veto.step_seconds,
            },
            translations: t.translations,
            created_at: t.created_at,
        }
    }
}
