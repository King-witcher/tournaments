//! A fase em jogo: configuração (`tournament_stages`), times (pelo seed),
//! séries decididas e, no suíço, os pareamentos gravados (`rounds`) → fase de
//! qualquer formato (`StageBracket`), sempre por `replay`. Nada do estado do
//! `engine` é gravado. Os mapas das séries e o veto de cada partida (também
//! refeito por replay) vêm junto.
//!
//! Só existe a primeira fase. Outras fases entrariam aqui: cada uma com a
//! classificação da anterior como participantes, montada quando a anterior
//! termina, e os resultados já separados por `stage`.

use engine::{
    champions::NoRepeatRule,
    core::{EngineError, MatchId, MatchState, ParticipantId, SeriesFrags, StageReport},
    formats::{self as ef, StageBracket, StageHistory, swiss::SwissRound},
    veto::VetoRules,
};
use sqlx::PgConnection;

use super::{
    changes, db,
    games::StoredGame,
    result::StoredResult,
    rounds,
    veto::{self, Champion, MatchVeto, VetoRow},
};
use crate::{
    shared::ApiError,
    teams::{self, StoredTeam},
    tournaments::{self, PoolMap, StoredTournament, TournamentStatus},
};

/// A primeira (e por enquanto única) fase.
pub const FIRST_STAGE: i16 = 1;

pub struct Stage {
    pub number: i16,
    pub format: ef::StageFormat,
    /// Os times da chave, na ordem dos seeds.
    pub entrants: Vec<StoredTeam>,
    /// As séries decididas da fase, na ordem do replay.
    pub results: Vec<StoredResult>,
    /// Os mapas enviados da fase, por partida e na ordem.
    pub games: Vec<StoredGame>,
    /// Suíço: os pareamentos gravados, da rodada 1 em diante; vazio nos
    /// outros formatos.
    pub rounds: Vec<SwissRound>,
    pub bracket: StageBracket,
    /// O veto das partidas: as regras do torneio, o tempo de cada passo, a
    /// regra de não repetição, os campeões do torneio (só com a fase de
    /// campeões), o pool de mapas e o veto refeito de cada partida que tem
    /// algum evento.
    pub veto_rules: VetoRules,
    pub veto_seconds: i32,
    pub no_repeat: Option<NoRepeatRule>,
    pub champions: Vec<Champion>,
    pub map_pool: Vec<PoolMap>,
    pub vetoes: Vec<MatchVeto>,
}

impl Stage {
    /// O time `id` da chave.
    pub fn entrant(&self, id: ParticipantId) -> Option<&StoredTeam> {
        self.entrants.iter().find(|e| e.id == id.0)
    }

    /// O time da chave em que `user_id` joga. "Jogador da partida" é membro do
    /// time de um dos lados: é por aqui que as ações dos jogadores (envio,
    /// resposta, veto, screenshot) acham o lado de quem age. Nos times, o que
    /// for só do capitão confere `StoredTeam::captain` depois.
    pub fn participant_of(&self, user_id: i64) -> Option<ParticipantId> {
        self.entrants
            .iter()
            .find(|e| e.has(user_id))
            .map(StoredTeam::participant)
    }

    pub fn result(&self, id: MatchId) -> Option<&StoredResult> {
        self.results.iter().find(|r| r.match_id == id)
    }

    /// Os mapas da partida `id`, na ordem.
    pub fn games_of(&self, id: MatchId) -> Vec<&StoredGame> {
        self.games.iter().filter(|g| g.match_id == id).collect()
    }

    /// O veto da partida `id`, se tem algum evento.
    pub fn veto_of(&self, id: MatchId) -> Option<&MatchVeto> {
        self.vetoes.iter().find(|v| v.match_id == id)
    }

    /// O que entra no replay, com outros resultados (a correção confere a
    /// fase com a série corrigida antes de gravar).
    pub fn history<'a>(&'a self, results: &'a [StoredResult]) -> History<'a> {
        History {
            entrants: &self.entrants,
            results,
            games: &self.games,
            rounds: &self.rounds,
        }
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

/// O que entra no replay da fase, além da configuração.
pub struct History<'a> {
    /// Os times, na ordem dos seeds.
    pub entrants: &'a [StoredTeam],
    pub results: &'a [StoredResult],
    /// Os mapas da fase, para os frags de cada série.
    pub games: &'a [StoredGame],
    /// Suíço: os pareamentos gravados.
    pub rounds: &'a [SwissRound],
}

/// Refaz a fase com os resultados confirmados de `history`, na ordem (com os
/// frags somados dos mapas de cada série), os pareamentos gravados e quem
/// está fora (desistiu ou saiu da comunidade: o suíço não o pareia mais), e
/// confere que cada resultado caiu na partida com os lados com que foi
/// gravado.
pub fn replay(
    format: ef::StageFormat,
    history: &History<'_>,
) -> Result<StageBracket, Contradiction> {
    let seeded = history
        .entrants
        .iter()
        .map(StoredTeam::participant)
        .collect();
    let mut confirmed: Vec<&StoredResult> = history.results.iter().collect();
    confirmed.sort_by_key(|r| r.position);
    let reports = confirmed
        .iter()
        .map(|r| StageReport {
            report: r.report(),
            frags: frags(r, history.games),
        })
        .collect();
    let out = history.entrants.iter().filter(|e| e.out());
    let stage_history = StageHistory {
        reports,
        rounds: history.rounds.to_vec(),
        out: out.map(StoredTeam::participant).collect(),
    };
    let bracket =
        StageBracket::replay(format, seeded, stage_history).map_err(Contradiction::Engine)?;
    for r in confirmed {
        match bracket.get(r.match_id).map(|m| m.state) {
            Some(MatchState::Completed { a, b, .. }) if (a, b) == (r.side_a, r.side_b) => {}
            _ => return Err(Contradiction::SidesChanged(r.match_id)),
        }
    }
    Ok(bracket)
}

/// Os frags da série decidida `r`, somados dos mapas dela (do vencedor e do
/// perdedor da série). `None` no W.O. (sem mapas que valham) e quando algum
/// mapa veio sem frags: a série fica sem frags, em vez de contar 0.
fn frags(r: &StoredResult, games: &[StoredGame]) -> Option<SeriesFrags> {
    r.score?;
    let mut sum: Option<SeriesFrags> = None;
    for g in games.iter().filter(|g| g.match_id == r.match_id) {
        let (a, b) = g.frags?;
        let (won, lost) = if r.winner == g.side_a { (a, b) } else { (b, a) };
        let total = sum.get_or_insert_default();
        total.winner += u32::try_from(won).ok()?;
        total.loser += u32::try_from(lost).ok()?;
    }
    sum
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

/// A fase em jogo, refeita. 404 `bracket.not_found` antes de o torneio
/// começar.
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
    let entrants = teams::seeded(conn, tournament.id).await?;
    let results = db::results(conn, tournament.id, number).await?;
    let games = db::games(conn, tournament.id, number).await?;
    let format = *config(tournament, number)?;
    let rounds = match format {
        ef::StageFormat::Swiss(_) => db::pairings(conn, tournament.id, number).await?,
        _ => Vec::new(),
    };
    let veto_rows = veto::events(conn, tournament.id, number).await?;
    let veto_rules = tournament
        .setup
        .veto
        .rules(tournament.setup.champion_rules.bans_per_game);
    let champions = if veto_rules.champion_phase {
        veto::champions(conn).await?
    } else {
        Vec::new()
    };
    let history = History {
        entrants: &entrants,
        results: &results,
        games: &games,
        rounds: &rounds,
    };
    let bracket = replay(format, &history).map_err(|err| {
        let id = tournament.id.get();
        match err {
            Contradiction::Engine(e) => tracing::error!(%e, id, "histórico gravado não fecha"),
            Contradiction::SidesChanged(m) => {
                tracing::error!(?m, id, "resultado gravado com outros lados");
            }
        }
        ApiError::internal()
    })?;
    let vetoes = vetoes(&bracket, &veto_rules, veto_rows)?;
    Ok(Stage {
        number,
        format,
        entrants,
        results,
        games,
        rounds,
        bracket,
        veto_rules,
        veto_seconds: tournament.setup.veto.step_seconds,
        no_repeat: tournament.setup.champion_rules.no_repeat,
        champions,
        map_pool: tournament.map_pool.clone(),
        vetoes,
    })
}

/// O veto de cada partida refeito das linhas `rows` (por partida, na ordem).
fn vetoes(
    bracket: &StageBracket,
    rules: &VetoRules,
    rows: Vec<VetoRow>,
) -> Result<Vec<MatchVeto>, ApiError> {
    let mut by_match: Vec<(MatchId, Vec<VetoRow>)> = Vec::new();
    for row in rows {
        match by_match.last_mut() {
            Some((id, group)) if *id == row.match_id => group.push(row),
            _ => by_match.push((row.match_id, vec![row])),
        }
    }
    by_match
        .into_iter()
        .map(|(id, rows)| {
            let best_of = bracket
                .get(id)
                .map(|m| m.best_of)
                .ok_or_else(ApiError::internal)?;
            let sides = rows[0].sides;
            MatchVeto::replay(rules.clone(), best_of, id, sides, rows)
        })
        .collect()
}

/// Gera a fase ao iniciar, no torneio travado, com `seeded` (os elegíveis na
/// ordem dos seeds, que já ficaram gravados): confere de novo a configuração
/// com o número real de participantes e o pool de campeões de agora, monta a
/// fase (no suíço, grava o pareamento da primeira rodada) e avisa os lados
/// das partidas que já nascem prontas. O resto sai do replay. Erros: os de
/// `Setup::check_entrants` (ex.: 422 `champions.pool_exhausted` `{required,
/// available}`) e os de montar a fase.
pub async fn start(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    seeded: &[ParticipantId],
) -> Result<(), ApiError> {
    let pool = tournaments::champion_pool(&mut *conn).await?;
    tournament
        .setup
        .check_entrants(tournament.mode, pool, seeded.len())?;
    let format = *config(tournament, FIRST_STAGE)?;
    let at = (tournament.tenant, tournament.id, FIRST_STAGE);
    let bracket = rounds::first(conn, at, format, seeded).await?;
    let entrants = teams::seeded(conn, tournament.id).await?;
    changes::started(conn, tournament, &bracket, &entrants).await
}

/// Descarta a fase ao desfazer o início: apaga séries, mapas, histórico e
/// pareamentos. Devolve quantas partidas tinham resultado ou mapa.
pub async fn discard(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
) -> Result<u64, ApiError> {
    Ok(db::discard(conn, tournament.id).await?)
}

/// Encerra o torneio quando a fase termina e o reabre quando uma correção
/// deixa partida por jogar. Devolve o status que vale.
pub async fn sync_status(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    stage: &Stage,
) -> Result<TournamentStatus, ApiError> {
    Ok(tournaments::sync_finished(conn, tournament.id, stage.bracket.is_finished()).await?)
}
