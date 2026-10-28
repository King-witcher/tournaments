//! A fase em jogo: configuração (`tournament_stages`), participantes (pelo
//! seed) e séries decididas → chave, sempre por `replay`. Nada do estado do
//! `engine` é gravado. Os mapas das séries vêm junto.
//!
//! Só existe a primeira fase. Outras fases entrariam aqui: cada uma com a
//! classificação da anterior como participantes, montada quando a anterior
//! termina, e os resultados já separados por `stage`.

use engine::{
    core::{EngineError, MatchId, MatchState, ParticipantId},
    formats::{self as ef, elimination::EliminationBracket},
};
use sqlx::PgConnection;

use super::{changes, db, games::StoredGame, result::StoredResult};
use crate::{
    participants::{self, Seeded},
    shared::ApiError,
    tournaments::{self, StoredTournament, TournamentStatus},
};

/// A primeira (e por enquanto única) fase.
pub const FIRST_STAGE: i16 = 1;

pub struct Stage {
    pub number: i16,
    pub format: ef::StageFormat,
    /// Na ordem dos seeds.
    pub entrants: Vec<Seeded>,
    /// As séries decididas da fase, na ordem do replay.
    pub results: Vec<StoredResult>,
    /// Os mapas enviados da fase, por partida e na ordem.
    pub games: Vec<StoredGame>,
    pub bracket: EliminationBracket,
}

impl Stage {
    pub fn entrant(&self, id: ParticipantId) -> Option<&Seeded> {
        self.entrants.iter().find(|e| e.id == id.0)
    }

    pub fn participant_of(&self, user_id: i64) -> Option<ParticipantId> {
        self.entrants
            .iter()
            .find(|e| e.user_id == user_id)
            .map(|e| ParticipantId(e.id))
    }

    pub fn result(&self, id: MatchId) -> Option<&StoredResult> {
        self.results.iter().find(|r| r.match_id == id)
    }

    /// Os mapas da partida `id`, na ordem.
    pub fn games_of(&self, id: MatchId) -> Vec<&StoredGame> {
        self.games.iter().filter(|g| g.match_id == id).collect()
    }
}

/// Por que o histórico não fecha numa partida.
#[derive(Debug)]
pub enum Contradiction {
    /// O `engine` recusou o resultado (partida não pronta, vencedor fora).
    Engine(EngineError),
    /// A partida existe e está pronta, mas com outros lados que os gravados
    /// no resultado.
    SidesChanged(MatchId),
}

/// Refaz a chave com os resultados confirmados de `results`, na ordem, e
/// confere que cada um caiu na partida com os lados com que foi gravado.
pub fn replay(
    stage: &ef::StageFormat,
    entrants: &[Seeded],
    results: &[StoredResult],
) -> Result<EliminationBracket, Contradiction> {
    let ef::StageFormat::Elimination(config) = stage;
    let seeded = entrants.iter().map(|e| ParticipantId(e.id)).collect();
    let mut confirmed: Vec<&StoredResult> = results.iter().collect();
    confirmed.sort_by_key(|r| r.position);
    let reports = confirmed.iter().map(|r| r.report());
    let bracket =
        EliminationBracket::replay(*config, seeded, reports).map_err(Contradiction::Engine)?;
    for r in confirmed {
        match bracket.get(r.match_id).map(|m| m.state) {
            Some(MatchState::Completed { a, b, .. }) if (a, b) == (r.side_a, r.side_b) => {}
            _ => return Err(Contradiction::SidesChanged(r.match_id)),
        }
    }
    Ok(bracket)
}

/// A configuração da fase `number`.
fn config(tournament: &StoredTournament, number: i16) -> Result<&ef::StageFormat, ApiError> {
    let index = usize::try_from(number - 1).map_err(|_| ApiError::internal())?;
    tournament
        .setup
        .stages
        .get(index)
        .ok_or_else(ApiError::internal)
}

/// A fase em jogo, com a chave refeita. 404 `bracket.not_found` antes de o
/// torneio começar.
pub async fn load(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
) -> Result<Stage, ApiError> {
    if !matches!(
        tournament.status,
        TournamentStatus::InProgress | TournamentStatus::Finished
    ) {
        return Err(ApiError::not_found("bracket.not_found"));
    }
    let number = FIRST_STAGE;
    let entrants = participants::seeded(conn, tournament.id).await?;
    let results = db::results(conn, tournament.id, number).await?;
    let games = db::games(conn, tournament.id, number).await?;
    let format = *config(tournament, number)?;
    let bracket = replay(&format, &entrants, &results).map_err(|err| {
        let id = tournament.id.get();
        match err {
            Contradiction::Engine(e) => tracing::error!(%e, id, "histórico gravado não fecha"),
            Contradiction::SidesChanged(m) => {
                tracing::error!(?m, id, "resultado gravado com outros lados");
            }
        }
        ApiError::internal()
    })?;
    Ok(Stage {
        number,
        format,
        entrants,
        results,
        games,
        bracket,
    })
}

/// Gera a chave ao iniciar, no torneio travado, com `seeded` (os elegíveis na
/// ordem dos seeds, que já ficaram gravados): confere de novo a configuração
/// com o número real de participantes e o pool de campeões de agora, e monta
/// a chave, avisando os lados das partidas que já nascem prontas. Nada mais
/// é gravado, porque ela sai do replay. Erros: os de
/// `Setup::check_entrants` (ex.: 422 `champions.pool_exhausted` `{required,
/// available}`) e os de montar a chave.
pub async fn start(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    seeded: &[ParticipantId],
) -> Result<(), ApiError> {
    let pool = tournaments::champion_pool(&mut *conn).await?;
    tournament
        .setup
        .check_entrants(tournament.mode, pool, seeded.len())?;
    let ef::StageFormat::Elimination(config) = config(tournament, FIRST_STAGE)?;
    let bracket = EliminationBracket::new(*config, seeded.to_vec())?;
    let entrants = participants::seeded(conn, tournament.id).await?;
    changes::started(conn, tournament, &bracket, &entrants).await
}

/// Descarta a chave ao desfazer o início: apaga séries, mapas e histórico.
/// Devolve quantas partidas tinham resultado ou mapa.
pub async fn discard(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
) -> Result<u64, ApiError> {
    Ok(db::discard(conn, tournament.id).await?)
}

/// Encerra o torneio quando a chave termina e o reabre quando uma correção
/// deixa partida por jogar. Devolve o status que vale.
pub async fn sync_status(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    stage: &Stage,
) -> Result<TournamentStatus, ApiError> {
    Ok(tournaments::sync_finished(conn, tournament.id, stage.bracket.is_finished()).await?)
}
