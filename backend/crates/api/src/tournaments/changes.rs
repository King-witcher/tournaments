//! O que o torneio emite no outbox: a mudança (a comunidade, por causa da
//! lista; quem está na página do torneio; "Seus torneios" dos inscritos; e a
//! home da organização enquanto ele espera ser iniciado) e o aviso de check-in
//! aberto (inscritos). As partidas emitem pelo `matches`, a partir da chave.

use sqlx::PgConnection;

use super::db::{self, Audience, TournamentId};
use crate::{
    outbox::{self, Event, Notice, NoticeKind},
    tenants,
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

/// A mudança. Com o início marcado já passado e o torneio por iniciar, a
/// home da organização também recarrega: ele está na lista do que espera
/// ela, com quantos fizeram check-in (`GET /me/organizing`). Quem tira o
/// torneio dessa lista (iniciar, adiar) avisa por conta própria
/// (`tenants::organizing_changed`), porque depois da mudança ele não está
/// mais nela.
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
    outbox::emit(conn, Event::me(me)).await?;
    if a.start_due {
        tenants::organizing_changed(conn, a.tenant_id).await?;
    }
    Ok(())
}

/// O rascunho `from` da comunidade `tenant` (o slug) agora é `to`: as abas da
/// organização abertas nele vão para o endereço novo.
pub async fn moved(
    conn: &mut PgConnection,
    tenant: &str,
    from: &str,
    to: &str,
) -> Result<(), sqlx::Error> {
    let moved = Event::Moved {
        tenant: tenant.to_owned(),
        tournament: Some(from.to_owned()),
        to: to.to_owned(),
    };
    outbox::emit(conn, moved).await
}

/// O organizador abriu o check-in: a mudança e o aviso aos inscritos.
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
