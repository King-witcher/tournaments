//! O tempo de cada passo do veto. O prazo que a série mostra é o nominal
//! (`VetoLog::deadline`); a ação ainda vale até `GRACE` depois dele (o
//! clique que sai no último segundo atravessa a rede), e só então o `engine`
//! sorteia a escolha de quem deixou o tempo acabar (`random_choice`, pela
//! seed da série), gravada como `timeout` e com a hora em que o sorteio
//! valeu (prazo + `GRACE`): o passo seguinte conta dali, com o tempo inteiro,
//! como se alguém tivesse esperado. Na janela simultânea dos
//! times, o prazo é um só (`team_pick_seconds`) e, no fim, cada partida de
//! cada um que não escolheu recebe um campeão sorteado (`random_team_choice`),
//! um de cada vez, na ordem do elenco. A API aplica ao ler a série ou a chave
//! e antes de cada ação (`any_due`, `apply_due`), e o relógio do `realtime`
//! aplica nos torneios que ninguém leu (`clock`), dormindo até o próximo
//! prazo. O mesmo sorteio completa o draft quando o organizador lança a
//! partida 1 antes do fim dele (`draw_turn`, pelo `advance`). `draw_turn` e
//! `due_draws` só dizem o que gravar: a sala de veto (`veto_rooms`) usa os mesmos.

use chrono::{DateTime, Utc};
use engine::{
    core::MatchState,
    veto::{Action, Choice, Event, Turn, Veto},
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

/// Quanto a ação ainda vale depois do prazo de cada passo (e da janela
/// simultânea): o cronômetro mostra o prazo nominal, e o clique dado no fim
/// dele chega depois (do usuário, 01/11).
pub const GRACE: chrono::TimeDelta = chrono::TimeDelta::seconds(2);

/// Os vetos com passo esperando, com a hora em que o sorteio vale (o prazo
/// mais `GRACE`).
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
        .filter_map(|mv| {
            let deadline = mv.deadline(stage.veto_seconds, stage.team_pick_seconds)?;
            Some((mv, deadline + GRACE))
        })
}

/// Quanto dura a vez de `state`: a janela da escolha simultânea ou um passo.
fn seconds_of(state: &Veto, step_seconds: i32, team_pick_seconds: i32) -> chrono::Duration {
    let seconds = match state.turn() {
        Turn::Act {
            action: Action::TeamPick,
            ..
        } => team_pick_seconds,
        _ => step_seconds,
    };
    chrono::Duration::seconds(i64::from(seconds))
}

/// A hora do próximo sorteio de veto da fase (o prazo mais `GRACE`).
pub fn next_deadline(status: TournamentStatus, stage: &stage::Stage) -> Option<DateTime<Utc>> {
    waiting(status, stage).map(|(_, d)| d).min()
}

/// Há passo com o prazo (mais `GRACE`) vencido em `now`.
pub fn any_due(status: TournamentStatus, stage: &stage::Stage, now: DateTime<Utc>) -> bool {
    next_deadline(status, stage).is_some_and(|d| d <= now)
}

/// Sorteia e grava os passos vencidos em `now`, em sequência: cada sorteio
/// fica com a hora em que valeu (prazo + `GRACE`), e o prazo seguinte conta
/// dali (com a API parada, cada passo vencido soma a tolerância). Quem chama
/// trava o torneio e depois emite (`settle`). Devolve quantos passos gravou.
pub async fn apply_due(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    stage: &stage::Stage,
    now: DateTime<Utc>,
) -> Result<u64, ApiError> {
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
        let rows = due_draws(
            &mut state,
            &ctx,
            first,
            now,
            stage.veto_seconds,
            stage.team_pick_seconds,
        )?;
        written += rows.len() as u64;
        for row in &rows {
            db::insert(conn, &at, mv.sides, row).await?;
        }
    }
    Ok(written)
}

/// Os sorteios de `state` vencidos em `now`, em sequência, a partir do que vale
/// em `first` (o prazo mais `GRACE`): aplica cada um em `state` e devolve o que
/// gravar, cada um com a hora em que valeu.
pub fn due_draws(
    state: &mut Veto,
    ctx: &Ctx,
    first: DateTime<Utc>,
    now: DateTime<Utc>,
    step_seconds: i32,
    team_pick_seconds: i32,
) -> Result<Vec<NewVetoEvent>, ApiError> {
    let mut rows = Vec::new();
    let mut deadline = first;
    while deadline <= now {
        let drawn = draw_turn(state, ctx, Some(deadline))?;
        if drawn.is_empty() {
            // Ninguém mais tem opção: não há o que sortear.
            break;
        }
        rows.extend(drawn);
        deadline += seconds_of(state, step_seconds, team_pick_seconds) + GRACE;
    }
    Ok(rows)
}

/// Sorteia, como no fim do tempo, o que a vez de `state` espera agora (o
/// passo de cada lado da vez; na janela simultânea, cada partida de cada um
/// que falta, um de cada vez, na ordem de `missing`), aplica em `state` e
/// devolve o que gravar como `timeout`, com a hora `when` (`None`: agora).
/// Vazio sem vez ou sem opção.
pub(super) fn draw_turn(
    state: &mut Veto,
    ctx: &Ctx,
    when: Option<DateTime<Utc>>,
) -> Result<Vec<NewVetoEvent>, ApiError> {
    let Turn::Act {
        game,
        action,
        sides: acting,
        pick_for,
        ..
    } = state.turn()
    else {
        return Ok(Vec::new());
    };
    let recused = |e: engine::veto::VetoError| {
        tracing::error!(%e, "sorteio do veto recusado");
        ApiError::internal()
    };
    let mut rows = Vec::new();
    if action == Action::TeamPick {
        for m in state.missing() {
            let options = state.team_options(m.side, m.player, m.game, &ctx.engine());
            let Some(champion) = state.random_team_choice(m.side, m.player, m.game, &ctx.engine())
            else {
                continue;
            };
            let event = Event::TeamPick {
                side: m.side,
                player: m.player,
                game: m.game,
                champion,
                gave_way: options.gave_way.is_some(),
            };
            state.apply(event, Some(&ctx.engine())).map_err(recused)?;
            rows.push(NewVetoEvent {
                champion: Some(champion),
                player: Some(m.player.0),
                pick_game: Some(i16::from(m.game)),
                gave_way: options.gave_way.is_some(),
                timeout: true,
                at: when,
                ..NewVetoEvent::new(
                    Kind::Pick,
                    Some(i16::from(game)),
                    Some(api_side(m.side)),
                    None,
                )
            });
        }
        return Ok(rows);
    }
    for side in acting {
        let Some(choice) = state.random_choice(side, &ctx.engine()) else {
            break;
        };
        let event = Event::Step {
            side,
            action,
            choice,
        };
        state.apply(event, Some(&ctx.engine())).map_err(recused)?;
        let (map, champion) = match choice {
            Choice::Map(m) => (Some(m), None),
            Choice::Champion(c) => (None, Some(c)),
        };
        rows.push(NewVetoEvent {
            map,
            champion,
            pick_game: champion.and(pick_for).map(i16::from),
            timeout: true,
            at: when,
            ..NewVetoEvent::new(
                Kind::of(action),
                Some(i16::from(game)),
                Some(api_side(side)),
                None,
            )
        });
    }
    Ok(rows)
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
