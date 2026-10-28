//! SQL dos torneios: criar, editar e excluir aqui; leitura em `read`,
//! mudanças de status em `status`, traduções em `translations`, os
//! destinatários do tempo real em `audience` e o que o agendador procura em
//! `scheduled`.

mod audience;
mod read;
mod scheduled;
mod status;
mod translations;

use chrono::{DateTime, Utc};
use engine::formats as ef;
use sqlx::{PgConnection, PgExecutor, types::Json};

use super::{
    edit::Edit,
    schedule::Schedule,
    setup::{Setup, TournamentMode},
    status::TournamentStatus,
    texts::Texts,
    types::TournamentTranslation,
    visibility::Visibility,
};
use crate::{
    shared::{ApiError, Locale},
    tenants::TenantId,
};

pub use audience::{Audience, audience};
pub use read::{current_status, find, find_for_update, list, slugs_for_member};
pub use scheduled::{due_check_ins, mark_start_reminded, starts_to_remind};
pub use status::{mark_registration_reopened, publish, set_status, sync_finished};
pub use translations::{delete_translation, put_translation};

/// Id interno de um torneio. Só sai do banco por `find`/`find_for_update`,
/// que filtram pelo tenant; quem recebe um `TournamentId` já passou por lá.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TournamentId(i64);

impl TournamentId {
    pub const fn get(self) -> i64 {
        self.0
    }
}

pub struct StoredTournament {
    pub id: TournamentId,
    /// O tenant em que `find` achou o torneio.
    pub tenant: TenantId,
    pub slug: String,
    pub locale: Locale,
    pub texts: Texts,
    pub mode: TournamentMode,
    /// O que vale agora: `find` já abriu o check-in se a janela abriu.
    pub status: TournamentStatus,
    pub visibility: Visibility,
    pub setup: Setup,
    pub schedule: Schedule,
    /// Janela de contestação de resultado, em minutos.
    pub contest_minutes: i32,
    pub translations: Vec<TournamentTranslation>,
    /// Inscritos agora.
    pub participants: i64,
    /// Se o `viewer` passado a `find` está inscrito.
    pub registered: bool,
    /// Se o `viewer` passado a `find` fez check-in.
    pub checked_in: bool,
    pub created_at: DateTime<Utc>,
}

pub struct NewTournament<'a> {
    pub slug: &'a str,
    pub locale: Locale,
    pub texts: &'a Texts,
    pub mode: TournamentMode,
    pub visibility: Visibility,
    pub setup: &'a Setup,
    pub schedule: Schedule,
    pub contest_minutes: i32,
}

/// Campeões ativos: o pool de todo torneio, por enquanto.
pub async fn champion_pool<'e>(db: impl PgExecutor<'e>) -> Result<u32, ApiError> {
    let count = sqlx::query_scalar!(r#"select count(*) as "count!" from champions where active"#)
        .fetch_one(db)
        .await?;
    u32::try_from(count).map_err(|_| ApiError::internal())
}

/// Cria o torneio como rascunho, com as fases.
pub async fn create(
    conn: &mut PgConnection,
    tenant: TenantId,
    new: NewTournament<'_>,
) -> Result<TournamentId, ApiError> {
    let id = sqlx::query_scalar!(
        r#"insert into tournaments
               (tenant_id, slug, locale, name, description, rules, mode, visibility,
                max_participants, champion_rules, starts_at, check_in_minutes, contest_minutes)
           values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
           on conflict (tenant_id, slug) do nothing
           returning id"#,
        tenant.get(),
        new.slug,
        new.locale.as_str(),
        new.texts.name,
        new.texts.description,
        new.texts.rules,
        new.mode.as_str(),
        new.visibility.as_str(),
        new.setup.max_participants,
        Json(new.setup.champion_rules) as _,
        new.schedule.starts_at,
        new.schedule.check_in_minutes,
        new.contest_minutes
    )
    .fetch_optional(&mut *conn)
    .await?
    .ok_or(ApiError::conflict("tournament.slug_taken"))?;
    let id = TournamentId(id);
    insert_stages(conn, id, &new.setup.stages).await?;
    Ok(id)
}

async fn insert_stages(
    conn: &mut PgConnection,
    id: TournamentId,
    stages: &[ef::StageFormat],
) -> Result<(), sqlx::Error> {
    for (position, stage) in (1..).zip(stages) {
        sqlx::query!(
            "insert into tournament_stages (tournament_id, position, config) values ($1, $2, $3)",
            id.0,
            position as i16,
            Json(stage) as _
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

/// Grava o PATCH: os textos e, quando vieram, a visibilidade, a configuração
/// (vagas, campeões e fases), a data e a janela de check-in, a janela de
/// contestação e o status.
pub async fn update(
    conn: &mut PgConnection,
    id: TournamentId,
    edit: &Edit,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"update tournaments
           set name = $2, description = $3, rules = $4, visibility = coalesce($5, visibility),
               status = coalesce($6, status), contest_minutes = coalesce($7, contest_minutes),
               updated_at = now()
           where id = $1"#,
        id.0,
        edit.texts.name,
        edit.texts.description,
        edit.texts.rules,
        edit.visibility.map(Visibility::as_str),
        edit.status.map(TournamentStatus::as_str),
        edit.contest_minutes
    )
    .execute(&mut *conn)
    .await?;
    if let Some(schedule) = &edit.schedule {
        sqlx::query!(
            "update tournaments set starts_at = $2, check_in_minutes = $3 where id = $1",
            id.0,
            schedule.starts_at,
            schedule.check_in_minutes
        )
        .execute(&mut *conn)
        .await?;
    }
    let Some(setup) = &edit.setup else {
        return Ok(());
    };
    sqlx::query!(
        "update tournaments set max_participants = $2, champion_rules = $3 where id = $1",
        id.0,
        setup.max_participants,
        Json(setup.champion_rules) as _
    )
    .execute(&mut *conn)
    .await?;
    sqlx::query!(
        "delete from tournament_stages where tournament_id = $1",
        id.0
    )
    .execute(&mut *conn)
    .await?;
    insert_stages(conn, id, &setup.stages).await
}

/// Exclui o torneio se ainda for rascunho. `false` se não era.
pub async fn delete_draft(conn: &mut PgConnection, id: TournamentId) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "delete from tournaments where id = $1 and status = 'draft'",
        id.0
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected() > 0)
}
