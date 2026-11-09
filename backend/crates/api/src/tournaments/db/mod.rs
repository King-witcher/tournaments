//! SQL dos torneios: criar, editar e excluir aqui; leitura em `read`,
//! mudanças de status em `status`, traduções em `translations`, as imagens
//! das regras e quem as usa em `rules_images`, os
//! destinatários do tempo real em `audience` e a tradução entre as colunas e
//! os tipos do `engine` em `columns`.

mod audience;
mod columns;
mod read;
mod rules_images;
mod stages;
mod status;
mod translations;

use chrono::{DateTime, Utc};
use engine::{core::MapId, stages::StageSetup};
use sqlx::{PgConnection, PgExecutor};

use self::columns::ChampionColumns;
use super::{
    admission::Admission,
    edit::Edit,
    schedule::Schedule,
    setup::Setup,
    status::TournamentStatus,
    texts::Texts,
    types::{StageStatus, TournamentTranslation},
    veto::PoolMap,
};
use crate::{
    shared::{ApiError, Locale},
    tenants::{self, TenantId},
};

pub use audience::{Address, Audience, audience};
pub use columns::{StageProgress, StageRow};
pub use read::{Listed, find, find_for_update, list, list_in, slugs_for_member};
pub use rules_images::{
    delete_unused_rules_images, lock_rules_images, register_rules_image, rules_images_with,
    set_rules_images, unused_rules_images,
};
pub use stages::{
    mark_stage_finished, mark_stage_started, open_stage_check_in, reset_stages,
    unmark_stage_started,
};
pub use status::{publish, set_status, sync_finished};
pub use translations::{delete_translation, put_translation};

/// Id interno de um torneio. Só sai do banco por `find`/`find_for_update` e
/// `list_in`, que filtram pelo tenant; quem recebe um `TournamentId` já passou
/// por lá.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TournamentId(i64);

impl TournamentId {
    pub const fn get(self) -> i64 {
        self.0
    }
}

pub struct StoredTournament {
    pub id: TournamentId,
    /// O tenant em que `find` achou o torneio, e o slug dele.
    pub tenant: TenantId,
    pub tenant_slug: String,
    pub slug: String,
    pub locale: Locale,
    pub texts: Texts,
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
    /// Quando o torneio foi iniciado (nulo antes do início).
    pub started_at: Option<DateTime<Utc>>,
    /// O objeto da capa no bucket público (nulo: sem capa).
    pub cover_key: Option<String>,
    /// Onde cada fase de `setup.stages` está, na mesma ordem.
    pub stage_progress: Vec<StageProgress>,
}

impl StoredTournament {
    /// Quantas fases o torneio tem.
    pub fn stage_count(&self) -> i16 {
        i16::try_from(self.setup.stages.len()).unwrap_or(i16::MAX)
    }

    /// A fase `number` (a partir de 1), se existe.
    pub fn stage(&self, number: i16) -> Option<(&StageSetup, StageProgress)> {
        let index = usize::try_from(number.checked_sub(1)?).ok()?;
        let setup = self.setup.stages.get(index)?;
        Some((
            setup,
            self.stage_progress.get(index).copied().unwrap_or_default(),
        ))
    }

    /// A fase `number` começou (a 1, com o torneio).
    pub fn stage_started(&self, number: i16) -> bool {
        match self.stage(number) {
            Some((_, p)) => p.started_at.is_some() || (number == 1 && self.status.has_started()),
            None => false,
        }
    }

    /// A fase atual: a última que começou (1 se nenhuma).
    pub fn current_stage(&self) -> i16 {
        (2..=self.stage_count())
            .rev()
            .find(|&n| self.stage_started(n))
            .unwrap_or(1)
    }

    /// Fases começadas (0 antes do início do torneio).
    pub fn started_stages(&self) -> usize {
        if self.status.has_started() {
            usize::try_from(self.current_stage()).unwrap_or(1)
        } else {
            0
        }
    }

    /// Como a fase `number` está, para a leitura do torneio.
    pub fn stage_status(&self, number: i16) -> StageStatus {
        let Some((_, p)) = self.stage(number) else {
            return StageStatus::Pending;
        };
        if self.stage_started(number) {
            if p.finished_at.is_some() {
                StageStatus::Finished
            } else {
                StageStatus::InProgress
            }
        } else if p.check_in_opened_at.is_some() {
            StageStatus::CheckIn
        } else {
            StageStatus::Pending
        }
    }
    /// O endereço e o nome, para avisar sem ler de novo (`Address`).
    pub fn address(&self) -> Address {
        Address {
            tenant: self.tenant_slug.clone(),
            slug: self.slug.clone(),
            name: self.texts.name.clone(),
        }
    }
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
    let champions = ChampionColumns::from(new.setup.champion_rules);
    let invite_code = tenants::free_invite_code(&mut *conn).await?;
    let id = sqlx::query_scalar!(
        r#"insert into tournaments
               (tenant_id, slug, locale, name, description, rules, mode, admission,
                max_participants, bans_per_series, mirror, same_team, no_repeat_who,
                no_repeat_scope, starts_at, check_in, contest_minutes,
                counts_for_rating, map_mode, champion_phase, veto_seconds, screenshots_required,
                team_size, team_pick_seconds, veto_order, pick_order, invite_token,
                veto_by, ban_by, pick_by, report_by)
           values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18,
                   $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31)
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
        champions.bans_per_series,
        champions.mirror,
        champions.same_team,
        champions.no_repeat_who,
        champions.no_repeat_scope,
        new.schedule.starts_at,
        new.schedule.check_in,
        new.contest_minutes,
        new.counts_for_rating,
        new.setup.veto.map_mode.as_str(),
        new.setup.veto.champion_phase,
        new.setup.veto.step_seconds,
        new.screenshots_required,
        new.setup.team_size as i16,
        new.setup.veto.team_pick_seconds,
        new.setup.veto.order.as_str(),
        new.setup.veto.pick_order.as_str(),
        invite_code,
        new.setup.team_rules.veto.as_str(),
        new.setup.team_rules.ban.as_str(),
        new.setup.team_rules.pick.as_str(),
        new.setup.team_rules.report.as_str()
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
    stages: &[StageSetup],
) -> Result<(), sqlx::Error> {
    for (position, stage) in (1..).zip(stages) {
        insert_stage(conn, id, position, *stage).await?;
    }
    Ok(())
}

async fn insert_stage(
    conn: &mut PgConnection,
    id: TournamentId,
    position: i16,
    stage: StageSetup,
) -> Result<(), sqlx::Error> {
    let row = StageRow::from(stage);
    sqlx::query!(
        "insert into tournament_stages
             (tournament_id, position, format, best_of, finals_best_of, bracket,
              grand_final_reset, rounds, tiebreakers, final_tie, advance, check_in)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        id.0,
        position,
        row.format,
        row.best_of,
        row.finals_best_of,
        row.bracket,
        row.grand_final_reset,
        row.rounds,
        row.tiebreakers.as_deref(),
        row.final_tie,
        row.advance,
        row.check_in
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Grava as fases `stages` no lugar das de agora (`existing` delas): muda só
/// as que mudaram (a fase começada vem igual, e o banco recusa a mudança),
/// acrescenta as novas (só antes do início) e apaga as do fim que saíram, com
/// os check-ins delas. Desligar o check-in de uma fase fecha o dela e apaga as
/// presenças confirmadas.
async fn update_stages(
    conn: &mut PgConnection,
    id: TournamentId,
    existing: usize,
    stages: &[StageSetup],
) -> Result<(), sqlx::Error> {
    for (position, stage) in (1..).zip(stages) {
        if usize::try_from(position).unwrap_or(usize::MAX) > existing {
            insert_stage(conn, id, position, *stage).await?;
            continue;
        }
        let row = StageRow::from(*stage);
        let done = sqlx::query!(
            r#"update tournament_stages
               set format = $3, best_of = $4, finals_best_of = $5, bracket = $6,
                   grand_final_reset = $7, rounds = $8, tiebreakers = $9, final_tie = $10,
                   advance = $11, check_in = $12,
                   check_in_opened_at = case when $12 then check_in_opened_at end
               where tournament_id = $1 and position = $2
                 and (format, best_of, finals_best_of, bracket, grand_final_reset, rounds,
                      tiebreakers, final_tie, advance, check_in)
                     is distinct from ($3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"#,
            id.0,
            position,
            row.format,
            row.best_of,
            row.finals_best_of,
            row.bracket,
            row.grand_final_reset,
            row.rounds,
            row.tiebreakers.as_deref(),
            row.final_tie,
            row.advance,
            row.check_in
        )
        .execute(&mut *conn)
        .await?;
        if done.rows_affected() > 0 && !stage.check_in {
            sqlx::query!(
                "delete from stage_teams
                 where tournament_id = $1 and stage = $2 and seed is null",
                id.0,
                position
            )
            .execute(&mut *conn)
            .await?;
        }
    }
    sqlx::query!(
        "delete from tournament_stages where tournament_id = $1 and position > $2",
        id.0,
        i16::try_from(stages.len()).unwrap_or(i16::MAX)
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Grava o PATCH: os textos e, quando vieram, a inscrição (`admission`), a configuração
/// (modo, vagas, campeões, fases e veto), a data e o check-in, a
/// janela de contestação e o status. Trocar o modo ou o tamanho refaz os
/// times no banco (trigger `tournaments_mode_teams`). As fases mudam no lugar
/// (`existing_stages`: quantas há agora), as que não começaram também depois
/// do início.
pub async fn update(
    conn: &mut PgConnection,
    id: TournamentId,
    edit: &Edit,
    existing_stages: usize,
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
            "update tournaments set starts_at = $2, check_in = $3 where id = $1",
            id.0,
            schedule.starts_at,
            schedule.check_in
        )
        .execute(&mut *conn)
        .await?;
    }
    let Some(setup) = &edit.setup else {
        return Ok(());
    };
    if edit.stages_changed {
        update_stages(conn, id, existing_stages, &setup.stages).await?;
    }
    if !edit.rest_changed {
        return Ok(());
    }
    let champions = ChampionColumns::from(setup.champion_rules);
    sqlx::query!(
        r#"update tournaments
           set max_participants = $2, bans_per_series = $3, mirror = $4, same_team = $5,
               no_repeat_who = $6, no_repeat_scope = $7, map_mode = $8, champion_phase = $9,
               veto_seconds = $10, mode = $11, team_size = $12, team_pick_seconds = $13,
               veto_order = $14, pick_order = $15, veto_by = $16, ban_by = $17, pick_by = $18,
               report_by = $19
           where id = $1"#,
        id.0,
        setup.max_participants,
        champions.bans_per_series,
        champions.mirror,
        champions.same_team,
        champions.no_repeat_who,
        champions.no_repeat_scope,
        setup.veto.map_mode.as_str(),
        setup.veto.champion_phase,
        setup.veto.step_seconds,
        setup.mode.as_str(),
        setup.team_size as i16,
        setup.veto.team_pick_seconds,
        setup.veto.order.as_str(),
        setup.veto.pick_order.as_str(),
        setup.team_rules.veto.as_str(),
        setup.team_rules.ban.as_str(),
        setup.team_rules.pick.as_str(),
        setup.team_rules.report.as_str()
    )
    .execute(&mut *conn)
    .await?;
    sqlx::query!("delete from tournament_maps where tournament_id = $1", id.0)
        .execute(&mut *conn)
        .await?;
    insert_map_pool(conn, id, &setup.veto.pool).await
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

/// Troca o slug do torneio; o antigo fica livre. Quem chama confere que é
/// rascunho (o banco também). Aviso antigo de um torneio que voltou a rascunho
/// não se reescreve: quem o recebeu já saiu da inscrição e não vê rascunho, e
/// o link dá em "não encontrado" de qualquer forma. 409
/// `tournament.slug_taken`.
pub async fn rename(conn: &mut PgConnection, id: TournamentId, slug: &str) -> Result<(), ApiError> {
    let done = sqlx::query!(
        "update tournaments set slug = $2, updated_at = now() where id = $1",
        id.0,
        slug
    )
    .execute(conn)
    .await;
    match done {
        Ok(_) => Ok(()),
        Err(sqlx::Error::Database(e))
            if e.constraint() == Some("tournaments_tenant_id_slug_key") =>
        {
            Err(ApiError::conflict("tournament.slug_taken"))
        }
        Err(e) => Err(e.into()),
    }
}

/// Exclui o torneio. A cascata leva tudo o que está dentro (o trigger
/// `tournaments_path_forgotten`, também as notificações e os avisos pendentes
/// dele; os objetos dos buckets entram nas filas de exclusão), e os mapas
/// confirmados marcam o rating do tenant para recálculo.
pub async fn delete(conn: &mut PgConnection, id: TournamentId) -> Result<(), sqlx::Error> {
    sqlx::query!("delete from tournaments where id = $1", id.0)
        .execute(conn)
        .await?;
    Ok(())
}
