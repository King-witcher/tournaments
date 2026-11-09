//! O PATCH de um torneio, gravado: junta o corpo com o que está gravado
//! (`edit`), confere a configuração com o pool de campeões, grava e emite.
//! A rota fica no `lifecycle`: tirar as fases que faltavam com a atual já
//! terminada encerra o torneio, e o pódio desse encerramento é da chave
//! (`matches`), que este domínio não alcança.

use chrono::Utc;
use sqlx::PgConnection;

use super::{
    changes,
    db::{self, StoredTournament},
    edit, rules_images, screenshots,
    types::UpdateTournament,
    veto,
};
use crate::{
    shared::ApiError,
    tenants::{self, Organizer, TenantMember},
};

/// O torneio depois do PATCH.
pub struct Updated {
    /// Como ficou, lido de novo.
    pub tournament: StoredTournament,
    /// Sem fase depois da atual, já terminada (as que faltavam saíram): o
    /// torneio encerrou agora, e quem chama anuncia o pódio.
    pub finished: bool,
}

/// Grava o PATCH `body` no torneio `slug`, travado na transação de quem chama
/// (`storage`: o bucket dos screenshots está configurado; `cdn`: a base das
/// imagens das regras, `rules_images_cdn`). Erros: os de `edit::apply`, os da
/// checagem da configuração (`Setup::check`), os das imagens das regras e 404
/// `tournament.not_found`.
pub async fn update(
    tx: &mut PgConnection,
    member: &TenantMember<Organizer>,
    slug: &str,
    mut body: UpdateTournament,
    storage: bool,
    cdn: Option<&str>,
) -> Result<Updated, ApiError> {
    const NOT_FOUND: ApiError = ApiError::not_found("tournament.not_found");
    let viewer = Some(member.user.id);
    let current = db::find_for_update(&mut *tx, member.tenant, slug, viewer)
        .await?
        .ok_or(NOT_FOUND)?;
    let veto = match body.veto.take() {
        Some(input) => Some(veto::resolve(&mut *tx, input, Some(&current.setup.veto)).await?),
        None => None,
    };
    let screenshots_required = screenshots::edit(
        current.status,
        current.screenshots_required,
        body.screenshots_required,
        storage,
    )?;
    let stored = edit::Current {
        status: current.status,
        slug: &current.slug,
        texts: &current.texts,
        admission: current.admission,
        setup: &current.setup,
        schedule: current.schedule,
        contest_minutes: current.contest_minutes,
        counts_for_rating: current.counts_for_rating,
        started_stages: current.started_stages(),
        current_finished: current.status.has_started()
            && current
                .stage(current.current_stage())
                .is_some_and(|(_, p)| p.finished_at.is_some()),
    };
    let edit = edit::apply(stored, veto, body)?;
    let rules_changed = edit.texts.rules != current.texts.rules;
    if let Some(setup) = &edit.setup {
        let pool = db::champion_pool(&mut *tx).await?;
        setup.check(pool, current.participants)?;
    }
    db::update(&mut *tx, current.id, &edit, current.setup.stages.len()).await?;
    if rules_changed {
        let rules = &edit.texts.rules;
        rules_images::save(&mut *tx, member.tenant, current.id, None, rules, cdn).await?;
    }
    if edit.finishes {
        // Sem fase depois da atual, já terminada: o torneio encerra, com a
        // classificação dela.
        db::sync_finished(&mut *tx, current.id, true).await?;
    }
    if edit.stages_changed && current.status.has_started() {
        // "Fase N pronta para iniciar" pode ter saído da home da organização.
        tenants::organizing_changed(&mut *tx, member.tenant).await?;
    }
    if let Some(required) = screenshots_required {
        db::set_screenshots_required(&mut *tx, current.id, required).await?;
    }
    if let Some(slug) = &edit.slug {
        db::rename(&mut *tx, current.id, slug).await?;
        changes::moved(&mut *tx, &member.slug, &current.slug, slug).await?;
    }
    changes::changed(&mut *tx, current.id).await?;
    if current.schedule.start_due(current.status, Utc::now()) {
        // Adiado, sai do que espera a organização (se continua, o evento é o
        // mesmo de `changed` e entra uma vez).
        tenants::organizing_changed(&mut *tx, member.tenant).await?;
    }
    let slug = edit.slug.as_deref().unwrap_or(&current.slug);
    let tournament = db::find(&mut *tx, member.tenant, slug, viewer)
        .await?
        .ok_or_else(ApiError::internal)?;
    Ok(Updated {
        tournament,
        finished: edit.finishes,
    })
}
