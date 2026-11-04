//! Depois de toda mudança na chave (`settle`): a série que passou a ter
//! outros lados (ou nenhum) perde o veto, a que ficou com os dois lados prontos
//! sem ninguém clicar (o elenco mudou) começa, e a partida seguinte de cada série
//! pronta começa quando a série aceita mais uma (a anterior foi enviada),
//! com o mapa em que a anterior foi enviada. O draft de campeões nunca fica
//! pela metade: se o organizador lançou a partida 1 antes do fim dele, o que
//! falta sai pelo sorteio do tempo esgotado (`due::draw_turn`) antes de a
//! partida 2 começar. O começo do veto também fica aqui (`begin`), para o "Pronto"
//! e o organizador que força.

use engine::{
    core::{Match, MatchState, ParticipantId},
    series,
    veto::{Event, Veto},
};
use sqlx::PgConnection;

use super::{
    Ctx,
    db::{self, NewVetoEvent},
    due,
    state::{Kind, ev_side},
};
use crate::{
    matches::{
        changes,
        db::MatchRef,
        games::{StoredGame, series_games},
        key,
        result::Side,
        stage::Stage,
    },
    shared::ApiError,
    tournaments::StoredTournament,
};

/// O evento de começo do veto da série `m`: o seed melhor começa. 422
/// `champions.pool_exhausted` `{required, available}` com campeões ativos de menos
/// para os bans da série.
fn begin_event(
    stage: &Stage,
    m: &Match,
    sides: (ParticipantId, ParticipantId),
    actor: Option<i64>,
) -> Result<NewVetoEvent, ApiError> {
    let seed_of = |p: ParticipantId| stage.entrant(p).and_then(|e| e.seed).unwrap_or(i32::MAX);
    let first = if seed_of(sides.0) <= seed_of(sides.1) {
        Side::A
    } else {
        Side::B
    };
    let seed: u64 = rand::random();
    let ctx = Ctx::between(stage, m.id, sides);
    Veto::new(stage.veto_rules.clone(), m.best_of).apply(
        Event::Begin {
            seed,
            first: ev_side(first),
        },
        Some(&ctx.engine()),
    )?;
    Ok(NewVetoEvent {
        // O u64 gravado bit a bit num bigint.
        seed: Some(seed as i64),
        ..NewVetoEvent::new(Kind::Begin, Some(1), Some(first), actor)
    })
}

/// Grava o evento `event` de começo e avisa quem age. `actor` é o organizador que
/// forçou.
async fn write_begin(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    stage: &Stage,
    m: &Match,
    sides: (ParticipantId, ParticipantId),
    event: &NewVetoEvent,
) -> Result<(), ApiError> {
    let key = &key::format(m.id);
    let at = MatchRef {
        tenant: tournament.tenant,
        tournament: tournament.id,
        stage: stage.number,
        key,
    };
    db::insert(conn, &at, sides, event).await?;
    changes::veto_started(conn, tournament, stage, key, sides).await
}

/// Começa o veto da série `m`. Erros: os de `begin_event`.
pub(super) async fn begin(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    stage: &Stage,
    m: &Match,
    sides: (ParticipantId, ParticipantId),
    actor: Option<i64>,
) -> Result<(), ApiError> {
    let event = begin_event(stage, m, sides, actor)?;
    write_begin(conn, tournament, stage, m, sides, &event).await
}

/// Aplica e devolve se gravou alguma coisa (quem chama refaz a fase).
pub async fn advance(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    stage: &Stage,
) -> Result<bool, ApiError> {
    let mut changed = false;
    for mv in &stage.vetoes {
        let Some(m) = stage.bracket.get(mv.match_id) else {
            continue;
        };
        let key = key::format(m.id);
        let at = MatchRef {
            tenant: tournament.tenant,
            tournament: tournament.id,
            stage: stage.number,
            key: &key,
        };
        let sides = match m.state {
            MatchState::Ready { a, b } | MatchState::Completed { a, b, .. } => Some((a, b)),
            _ => None,
        };
        if sides != Some(mv.sides) {
            db::remove(conn, &at).await?;
            changed = true;
            continue;
        }
        let MatchState::Ready { a, b } = m.state else {
            continue;
        };
        if !mv.state.begun() {
            if stage.readiness(m.id, mv.sides).both() {
                // Sem quem clicou para receber o erro: fica no log, e o
                // organizador ainda pode resolver a série.
                match begin_event(stage, m, mv.sides, None) {
                    Ok(event) => {
                        write_begin(conn, tournament, stage, m, mv.sides, &event).await?;
                        changed = true;
                    }
                    Err(e) => tracing::warn!(code = e.code, key, "veto não começou"),
                }
            }
            continue;
        }
        let games = stage.games_of(m.id);
        if series::check_next(m.best_of, &series_games(&games), None).is_err() {
            continue;
        }
        let next = games.len() + 1;
        let mut state = mv.state.clone();
        let mut current = usize::from(state.game().unwrap_or(1));
        if current < next && current == 1 && stage.veto_rules.champion_phase {
            let ctx = Ctx::of(stage, mv);
            loop {
                let rows = due::draw_turn(&mut state, &ctx, None)?;
                if rows.is_empty() {
                    break;
                }
                for row in &rows {
                    db::insert(conn, &at, (a, b), row).await?;
                }
                changed = true;
            }
        }
        while current < next {
            let previous: &StoredGame = games[current - 1];
            let game = u8::try_from(current + 1).map_err(|_| ApiError::internal())?;
            let played_map = previous.map.as_ref().map(|m| m.id);
            state
                .apply(
                    Event::Game {
                        game,
                        previous: played_map,
                    },
                    None,
                )
                .map_err(|e| {
                    tracing::error!(%e, key, "partida seguinte do veto recusada");
                    ApiError::internal()
                })?;
            let event = NewVetoEvent {
                played_map,
                ..NewVetoEvent::new(Kind::Game, Some(i16::from(game)), None, None)
            };
            db::insert(conn, &at, (a, b), &event).await?;
            changed = true;
            current += 1;
        }
    }
    Ok(changed)
}
