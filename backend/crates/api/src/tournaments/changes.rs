//! O que o torneio emite no outbox: a mudança (a comunidade, por causa da
//! lista; quem está na página do torneio; e "Seus torneios" dos inscritos) e
//! os avisos de check-in aberto (inscritos) e de hora de iniciar (organizers e
//! owner). As partidas emitem pelo `matches`, a partir da chave.

use sqlx::PgConnection;

use super::db::{self, Audience, TournamentId};
use crate::{
    outbox::{self, Event, Notice, NoticeKind},
    tenants::{self, TenantId},
};

/// O torneio mudou (dados, status, inscrições, check-ins, seeding).
pub async fn changed(conn: &mut PgConnection, id: TournamentId) -> Result<(), sqlx::Error> {
    changed_with(conn, id, &[]).await
}

/// Como `changed`, com `also` em "Seus torneios": quem saiu dos inscritos
/// nesta mudança.
pub async fn changed_with(
    conn: &mut PgConnection,
    id: TournamentId,
    also: &[i64],
) -> Result<(), sqlx::Error> {
    let audience = db::audience(conn, id).await?;
    emit_changed(conn, &audience, also).await
}

async fn emit_changed(
    conn: &mut PgConnection,
    a: &Audience,
    also: &[i64],
) -> Result<(), sqlx::Error> {
    let tenant = a.tenant.clone();
    outbox::emit(conn, Event::Tenant { tenant }).await?;
    let changed = Event::Tournament {
        tenant: a.tenant.clone(),
        tournament: a.slug.clone(),
        match_key: None,
    };
    outbox::emit(conn, changed).await?;
    let me = a.participants.iter().chain(also).copied();
    outbox::emit(conn, Event::me(me)).await
}

/// O check-in abriu (pelo organizador, pela hora ao ler ou pelo agendador):
/// a mudança e o aviso aos inscritos.
pub async fn check_in_opened(conn: &mut PgConnection, id: TournamentId) -> Result<(), sqlx::Error> {
    let a = db::audience(conn, id).await?;
    emit_changed(conn, &a, &[]).await?;
    let notice = Notice {
        kind: NoticeKind::CheckInOpened {
            tournament: a.name.clone(),
        },
        path: a.path(),
    };
    outbox::emit(conn, Event::notice(a.participants.iter().copied(), notice)).await
}

/// O início marcado chegou sem o torneio iniciar: aviso para organizers e
/// owner.
pub async fn start_due(
    conn: &mut PgConnection,
    tenant: TenantId,
    id: TournamentId,
) -> Result<(), sqlx::Error> {
    let a = db::audience(conn, id).await?;
    let staff = tenants::staff(conn, tenant).await?;
    let path = a.path();
    let notice = Notice {
        kind: NoticeKind::StartDue { tournament: a.name },
        path,
    };
    outbox::emit(conn, Event::notice(staff, notice)).await
}
