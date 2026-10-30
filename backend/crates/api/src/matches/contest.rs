//! Quem contesta uma partida e até quando (o envio e as respostas ficam em
//! `report`).
//!
//! - Pendente: qualquer jogador da série menos quem a enviou, também o colega
//!   que discorda do que o próprio time enviou. Confirmar é só do outro time.
//! - Confirmada pelo adversário ou pelo fim do prazo (a contestação tardia):
//!   as mesmas pessoas, até a próxima série de um dos dois lados começar
//!   (veto começado ou partida enviada depois do envio desta), com o torneio
//!   em andamento. A partida volta a contestada e, se a série estava decidida
//!   por ela, a série reabre e a chave é refeita (`correction`). Fica fechada
//!   quando reabrir contradiz uma série decidida depois (um W.O. na seguinte,
//!   por exemplo), quando a série acabou em W.O. e quando a organização
//!   confirmou ou lançou a partida (ela já olhou).

use engine::{core::Match, series};

use super::{
    correction::{self, Change},
    games::{GameStatus, StoredGame, series_games},
    result::Confirmation,
    stage::Stage,
    veto,
};
use crate::{shared::ApiError, tournaments::TournamentStatus};

/// 409: a contestação tardia já não vale.
pub const CLOSED: ApiError = ApiError::conflict("match.contest_closed");

/// `user_id` pode contestar agora a partida `g` da série `m`.
pub fn allowed(
    user_id: i64,
    status: TournamentStatus,
    stage: &Stage,
    m: &Match,
    g: &StoredGame,
) -> bool {
    let plays = stage
        .participant_of(user_id)
        .and_then(|p| g.side_of(p))
        .is_some();
    if !plays || g.reported_by == Some(user_id) || status != TournamentStatus::InProgress {
        return false;
    }
    match g.status {
        GameStatus::Pending => true,
        GameStatus::Contested => false,
        GameStatus::Confirmed => late(stage, m, g).is_ok(),
    }
}

/// A contestação tardia da partida confirmada `g` da série `m`, com o torneio
/// em andamento: o que ela muda na série decidida (`None` se a série não
/// estava decidida, ou continua decidida igual). Erro `CLOSED` se não vale
/// mais.
pub fn late(stage: &Stage, m: &Match, g: &StoredGame) -> Result<Option<Change>, ApiError> {
    let by_player = g.reported_side.is_some()
        && matches!(
            g.confirmed_via,
            Some(Confirmation::Opponent | Confirmation::Deadline)
        );
    let walkover = stage
        .result(m.id)
        .is_some_and(|r| r.confirmed_via != Confirmation::Maps);
    if g.status != GameStatus::Confirmed || !by_player || walkover || moved_on(stage, g) {
        return Err(CLOSED);
    }
    let games = stage.games_of(m.id);
    let mut after = series_games(&games);
    let index = games
        .iter()
        .position(|x| x.id == g.id)
        .ok_or_else(ApiError::internal)?;
    after[index].status = series::GameStatus::Contested;
    let decided = series::evaluate(m.best_of, &after).decided;
    let change = correction::series_change(stage.result(m.id), decided, g.sides());
    if let Some(change) = &change {
        correction::check(stage, m.id, change).map_err(|_| CLOSED)?;
    }
    Ok(change)
}

/// Um dos lados da partida `g` já foi para outra série depois do envio
/// dela: o veto começou ou uma partida foi enviada (ou lançada).
fn moved_on(stage: &Stage, g: &StoredGame) -> bool {
    let theirs = |(a, b)| [g.side_a, g.side_b].iter().any(|s| *s == a || *s == b);
    let played = stage
        .games
        .iter()
        .any(|o| o.match_id != g.match_id && theirs(o.sides()) && o.reported_at > g.reported_at);
    let vetoed = stage.vetoes.iter().any(|v| {
        v.match_id != g.match_id
            && theirs(v.sides)
            && veto::begun_at(stage, v.match_id).is_some_and(|at| at > g.reported_at)
    });
    played || vetoed
}
