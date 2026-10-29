//! O tempo de cada passo do veto. Vencido o prazo, o `engine` sorteia a
//! escolha de quem deixou o tempo acabar (`random_choice`, pela seed da
//! série), gravada como `timeout` e com a hora do prazo: o passo seguinte
//! conta dali, como se alguém tivesse esperado. A API aplica ao ler a
//! partida ou a chave e antes de cada ação (`any_due`, `apply_due`), e o
//! relógio do `realtime` aplica nos torneios que ninguém leu (`clock`),
//! dormindo até o próximo prazo.

use chrono::{DateTime, Utc};
use engine::{
    core::MatchState,
    veto::{Choice, Event, Turn},
};
use sqlx::{PgConnection, PgPool};

use super::{
    Ctx,
    db::{self, NewVetoEvent},
    state::{Kind, MatchVeto, api_side},
};
use crate::{
    matches::{changes::Snapshot, db::MatchRef, key, settle, stage},
    shared::ApiError,
    tournaments::{self, StoredTournament, TournamentStatus},
};

/// Os vetos com passo esperando, com o prazo de cada um.
fn waiting<'a>(
    status: TournamentStatus,
    stage: &'a stage::Stage,
) -> impl Iterator<Item = (&'a MatchVeto, DateTime<Utc>)> + 'a {
    stage
        .vetoes
        .iter()
        .filter(move |_| status == TournamentStatus::InProgress)
        .filter(|mv| {
            stage
                .bracket
                .get(mv.match_id)
                .is_some_and(|m| matches!(m.state, MatchState::Ready { .. }))
        })
        .filter_map(|mv| Some((mv, mv.deadline(stage.veto_seconds)?)))
}

/// O próximo prazo de veto da fase.
pub fn next_deadline(status: TournamentStatus, stage: &stage::Stage) -> Option<DateTime<Utc>> {
    waiting(status, stage).map(|(_, d)| d).min()
}

/// Há passo com o prazo vencido em `now`.
pub fn any_due(status: TournamentStatus, stage: &stage::Stage, now: DateTime<Utc>) -> bool {
    next_deadline(status, stage).is_some_and(|d| d <= now)
}

/// Sorteia e grava os passos vencidos em `now`, em sequência (cada prazo
/// conta do anterior). Quem chama trava o torneio e depois emite
/// (`settle`). Devolve quantos passos gravou.
pub async fn apply_due(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    stage: &stage::Stage,
    now: DateTime<Utc>,
) -> Result<u64, ApiError> {
    let seconds = chrono::Duration::seconds(i64::from(stage.veto_seconds));
    let mut written = 0;
    let due: Vec<(&MatchVeto, DateTime<Utc>)> = waiting(tournament.status, stage)
        .filter(|(_, d)| *d <= now)
        .collect();
    for (mv, first) in due {
        let ctx = Ctx::of(stage, mv);
        let key = key::format(mv.match_id);
        let at = MatchRef {
            tenant: tournament.tenant,
            tournament: tournament.id,
            stage: stage.number,
            key: &key,
        };
        let mut state = mv.state.clone();
        let mut deadline = first;
        while deadline <= now {
            let Turn::Act {
                game,
                action,
                sides,
                ..
            } = state.turn()
            else {
                break;
            };
            for side in sides {
                let Some(choice) = state.random_choice(side, &ctx.engine()) else {
                    break;
                };
                let event = Event::Step {
                    side,
                    action,
                    choice,
                };
                state.apply(event, Some(&ctx.engine())).map_err(|e| {
                    tracing::error!(%e, key, "sorteio do veto recusado");
                    ApiError::internal()
                })?;
                let (map, champion) = match choice {
                    Choice::Map(m) => (Some(m), None),
                    Choice::Champion(c) => (None, Some(c)),
                };
                let row = NewVetoEvent {
                    map,
                    champion,
                    timeout: true,
                    at: Some(deadline),
                    ..NewVetoEvent::new(
                        Kind::of(action),
                        Some(i16::from(game)),
                        Some(api_side(side)),
                        None,
                    )
                };
                db::insert(conn, &at, mv.sides, &row).await?;
                written += 1;
            }
            deadline += seconds;
        }
    }
    Ok(written)
}

/// O relógio: aplica os passos vencidos em `now` em todos os torneios com
/// veto aberto, cada um na própria transação e com o torneio travado (só
/// quando há algum vencido: ler não trava), e devolve o próximo prazo. O
/// erro de um torneio fica no log e não para os outros.
pub async fn clock(
    db: &PgPool,
    now: DateTime<Utc>,
) -> Result<(u64, Option<DateTime<Utc>>), ApiError> {
    let mut next: Option<DateTime<Utc>> = None;
    let mut written = 0;
    for (tenant, slug) in db::open_tournaments(db).await? {
        let pass = async {
            let mut conn = db.acquire().await?;
            let Some(t) = tournaments::find(&mut conn, tenant, &slug, None).await? else {
                return Ok((0, None));
            };
            let current = stage::load(&mut conn, &t).await?;
            if !any_due(t.status, &current, now) {
                return Ok((0, next_deadline(t.status, &current)));
            }
            drop(conn);
            let mut tx = db.begin().await?;
            let Some(t) = tournaments::find_for_update(&mut tx, tenant, &slug, None).await? else {
                return Ok((0, None));
            };
            let current = stage::load(&mut tx, &t).await?;
            let before = Snapshot::of(&current);
            let n = apply_due(&mut tx, &t, &current, now).await?;
            let (after, _) = settle::settle(&mut tx, &t, None, &before).await?;
            tx.commit().await?;
            Ok::<_, ApiError>((n, next_deadline(t.status, &after)))
        };
        match pass.await {
            Ok((n, deadline)) => {
                written += n;
                next = match (next, deadline) {
                    (Some(a), Some(b)) => Some(a.min(b)),
                    (a, b) => a.or(b),
                };
            }
            Err(e) => tracing::error!(slug, code = e.code, "tempo do veto falhou"),
        }
    }
    Ok((written, next))
}
