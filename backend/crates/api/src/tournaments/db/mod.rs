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
use engine::{core::MapId, formats as ef};
use sqlx::{PgConnection, PgExecutor, types::Json};

use super::{
    admission::Admission, edit::Edit, schedule::Schedule, setup::Setup, status::TournamentStatus,
    texts::Texts, types::TournamentTranslation, veto::PoolMap,
};
use crate::{
    shared::{ApiError, Locale},
    tenants::TenantId,
};

pub use audience::{Audience, audience};
pub use read::{current_status, find, find_for_update, list, slugs_for_member};
pub use scheduled::{due_check_ins, mark_start_reminded, starts_to_remind};
pub use status::{publish, set_status, sync_finished};
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
    /// O que vale agora: `find` já abriu o check-in se a janela abriu.
    pub status: TournamentStatus,
    pub admission: Admission,
    pub setup: Setup,
    pub schedule: Schedule,
    /// Janela de contestação de resultado, em minutos.
    pub contest_minutes: i32,
    /// Se os mapas contam para o rating da comunidade.
    pub counts_for_rating: bool,
    /// O pool de mapas (`setup.veto.pool`) com slug e nome, na ordem.
    pub map_pool: Vec<PoolMap>,
    /// O envio do jogador exige screenshot do placar (vale com o bucket
    /// configurado).
    pub screenshots_required: bool,
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
    pub admission: Admission,
    pub setup: &'a Setup,
    pub schedule: Schedule,
    pub contest_minutes: i32,
    pub counts_for_rating: bool,
    pub screenshots_required: bool,
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
               (tenant_id, slug, locale, name, description, rules, mode, admission,
                max_participants, champion_rules, starts_at, check_in_minutes, contest_minutes,
                counts_for_rating, map_mode, champion_phase, veto_seconds, screenshots_required,
                team_size, team_pick_seconds)
           values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18,
                   $19, $20)
           on conflict (tenant_id, slug) do nothing
           returning id"#,
        tenant.get(),
        new.slug,
        new.locale.as_str(),
        new.texts.name,
        new.texts.description,
        new.texts.rules,
        new.setup.mode.as_str(),
        new.admission.as_str(),
        new.setup.max_participants,
        Json(new.setup.champion_rules) as _,
        new.schedule.starts_at,
        new.schedule.check_in_minutes,
        new.contest_minutes,
        new.counts_for_rating,
        new.setup.veto.map_mode.as_str(),
        new.setup.veto.champion_phase,
        new.setup.veto.step_seconds,
        new.screenshots_required,
        new.setup.team_size as i16,
        new.setup.veto.team_pick_seconds
    )
    .fetch_optional(&mut *conn)
    .await?
    .ok_or(ApiError::conflict("tournament.slug_taken"))?;
    let id = TournamentId(id);
    insert_stages(conn, id, &new.setup.stages).await?;
    insert_map_pool(conn, id, &new.setup.veto.pool).await?;
    Ok(id)
}

async fn insert_map_pool(
    conn: &mut PgConnection,
    id: TournamentId,
    pool: &[MapId],
) -> Result<(), sqlx::Error> {
    let ids: Vec<i64> = pool.iter().map(|m| m.0).collect();
    sqlx::query!(
        r#"insert into tournament_maps (tournament_id, map_id, position)
           select $1, map_id, position::smallint
           from unnest($2::bigint[]) with ordinality as p(map_id, position)"#,
        id.0,
        &ids
    )
    .execute(conn)
    .await?;
    Ok(())
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

/// Grava o PATCH: os textos e, quando vieram, a inscrição (`admission`), a configuração
/// (modo, vagas, campeões, fases e veto), a data e a janela de check-in, a
/// janela de contestação e o status. Trocar o modo ou o tamanho refaz os
/// times no banco (trigger `tournaments_mode_teams`).
pub async fn update(
    conn: &mut PgConnection,
    id: TournamentId,
    edit: &Edit,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"update tournaments
           set name = $2, description = $3, rules = $4, admission = coalesce($5, admission),
               status = coalesce($6, status), contest_minutes = coalesce($7, contest_minutes),
               counts_for_rating = coalesce($8, counts_for_rating), updated_at = now()
           where id = $1"#,
        id.0,
        edit.texts.name,
        edit.texts.description,
        edit.texts.rules,
        edit.admission.map(Admission::as_str),
        edit.status.map(TournamentStatus::as_str),
        edit.contest_minutes,
        edit.counts_for_rating
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
        r#"update tournaments
           set max_participants = $2, champion_rules = $3, map_mode = $4, champion_phase = $5,
               veto_seconds = $6, mode = $7, team_size = $8, team_pick_seconds = $9
           where id = $1"#,
        id.0,
        setup.max_participants,
        Json(setup.champion_rules) as _,
        setup.veto.map_mode.as_str(),
        setup.veto.champion_phase,
        setup.veto.step_seconds,
        setup.mode.as_str(),
        setup.team_size as i16,
        setup.veto.team_pick_seconds
    )
    .execute(&mut *conn)
    .await?;
    sqlx::query!("delete from tournament_maps where tournament_id = $1", id.0)
        .execute(&mut *conn)
        .await?;
    insert_map_pool(conn, id, &setup.veto.pool).await?;
    sqlx::query!(
        "delete from tournament_stages where tournament_id = $1",
        id.0
    )
    .execute(&mut *conn)
    .await?;
    insert_stages(conn, id, &setup.stages).await
}

/// Liga ou desliga o screenshot obrigatório.
pub async fn set_screenshots_required(
    conn: &mut PgConnection,
    id: TournamentId,
    required: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update tournaments set screenshots_required = $2, updated_at = now() where id = $1",
        id.0,
        required
    )
    .execute(conn)
    .await?;
    Ok(())
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
