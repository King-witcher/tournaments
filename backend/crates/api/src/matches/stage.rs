//! A fase em jogo: configuração (`tournament_stages`), times (pelo seed),
//! séries decididas e, no suíço, os pareamentos gravados (`rounds`) → fase de
//! qualquer formato (`StageBracket`), sempre por `replay`. Nada do estado do
//! `engine` é gravado. Os mapas das séries e o veto de cada partida (também
//! refeito por replay) vêm junto. Duas idas ao banco: os times (`teams`) e o
//! resto (`db::stage_rows`, com os campeões quando há draft). Para a colocação
//! nos torneios encerrados, `brackets` refaz só a chave de vários torneios,
//! com as mesmas duas idas.
//!
//! Só existe a primeira fase. Outras fases entrariam aqui: cada uma com a
//! classificação da anterior como participantes, montada quando a anterior
//! termina, e os resultados já separados por `stage`.

use engine::{
    core::{EngineError, MatchId, MatchState, ParticipantId, PlayerId, SeriesFrags, StageReport},
    formats::{self as ef, StageBracket, StageHistory, swiss::SwissRound},
    veto::{Rosters, VetoRules},
};
use sqlx::PgConnection;

use super::{
    changes,
    db::{self, Parts},
    games::StoredGame,
    result::{Side, StoredResult},
    rounds,
    veto::{Champion, MatchVeto, MatchVetoRow, VetoLog},
};
use crate::{
    shared::ApiError,
    teams::{self, StoredTeam},
    tournaments::{self, PoolMap, StoredTournament, TeamRules, TournamentId, TournamentStatus},
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
    /// O veto das séries: as regras do torneio (com as de campeões), o tempo
    /// de cada passo, os campeões do torneio (só com o draft no site), o pool
    /// de mapas e o veto refeito de cada série que tem algum evento.
    pub veto_rules: VetoRules,
    pub veto_seconds: i32,
    /// TDM: a janela simultânea de campeões.
    pub team_pick_seconds: i32,
    pub champions: Vec<Champion>,
    pub map_pool: Vec<PoolMap>,
    pub vetoes: Vec<MatchVeto>,
    /// Quem age pelo time.
    pub team_rules: TeamRules,
    /// TDM com o draft: quem jogou cada partida enviada (o elenco do veto);
    /// vazio nos outros casos.
    lineups: Vec<(MatchId, i16, Side, i64)>,
}

/// O que uma ação gravou na fase, para `Stage::refresh` reler só isso.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wrote {
    /// Eventos de veto (pronto, começo, passo, desfazer, pedido e resposta).
    Veto,
    /// Partidas (envio, confirmação), com a escalação que o banco grava
    /// junto.
    Games,
}

impl Stage {
    /// O time `id` da chave.
    pub fn entrant(&self, id: ParticipantId) -> Option<&StoredTeam> {
        self.entrants.iter().find(|e| e.id == id.0)
    }

    /// O time da chave em que `user_id` joga. "Jogador da partida" é membro do
    /// time de um dos lados: é por aqui que as ações dos jogadores (envio,
    /// resposta, veto, screenshot) acham o lado de quem age; quem do time pode
    /// agir é a regra do torneio (`acting`).
    pub fn participant_of(&self, user_id: i64) -> Option<ParticipantId> {
        participant_of(&self.entrants, user_id)
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

    /// Os frags da série decidida `id` por lado (`a`, `b`), os mesmos que
    /// entram na tabela. `None` antes da decisão e nos casos de `frags`.
    pub fn series_frags(&self, id: MatchId) -> Option<(i32, i32)> {
        let sum = frags(self.result(id)?, &self.games)?;
        Some((sum.a, sum.b))
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

/// O time de `entrants` em que `user_id` joga (`Stage::participant_of`).
pub fn participant_of(entrants: &[StoredTeam], user_id: i64) -> Option<ParticipantId> {
    entrants
        .iter()
        .find(|e| e.has(user_id))
        .map(StoredTeam::participant)
}

/// Quem escolhe campeão pelo time `id` no TDM: os jogadores dele que estão
/// na comunidade, o capitão primeiro e depois na ordem de inscrição.
fn roster(entrants: &[StoredTeam], id: ParticipantId) -> Vec<PlayerId> {
    entrants
        .iter()
        .find(|e| e.id == id.0)
        .map(|t| {
            t.members
                .iter()
                .filter(|m| !m.left)
                .map(|m| PlayerId(m.user_id))
                .collect()
        })
        .unwrap_or_default()
}

/// TDM: quem jogou a partida `game` da série `id` por lado (`[a, b]`), de
/// `lineups` (vazio no duelo, em que o pick é do lado).
fn lineup(lineups: &[(MatchId, i16, Side, i64)], id: MatchId, game: usize) -> [Vec<PlayerId>; 2] {
    let mut sides = [Vec::new(), Vec::new()];
    for (m, number, side, user) in lineups {
        if *m == id && usize::try_from(*number).ok() == Some(game) {
            sides[usize::from(*side == Side::B)].push(PlayerId(*user));
        }
    }
    sides
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
/// frags somados dos mapas de cada série), os pareamentos gravados, o rating
/// de cada time no início (para o menor rating) e quem está fora (desistiu ou
/// saiu da comunidade: o suíço não o pareia mais), e confere que cada
/// resultado caiu na partida com os lados com que foi gravado.
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
        ratings: history
            .entrants
            .iter()
            .filter_map(|e| e.rating_at_start.map(|r| (e.participant(), r)))
            .collect(),
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

/// Os frags da série decidida `r`, somados dos mapas dela, por lado: os mapas
/// têm os mesmos lados da série (`check_match_series`). `None` no W.O. (sem
/// mapas que valham) e quando algum mapa veio sem frags: a série fica sem
/// frags, em vez de contar 0.
fn frags(r: &StoredResult, games: &[StoredGame]) -> Option<SeriesFrags> {
    r.score?;
    let mut sum: Option<SeriesFrags> = None;
    for g in games.iter().filter(|g| g.match_id == r.match_id) {
        let (a, b) = g.frags?;
        let total = sum.get_or_insert_default();
        total.a += i32::from(a);
        total.b += i32::from(b);
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

/// `replay`, com o erro no log: o histórico gravado do torneio `id` que não
/// fecha é um erro nosso (500).
fn replayed(
    id: TournamentId,
    format: ef::StageFormat,
    history: &History<'_>,
) -> Result<StageBracket, ApiError> {
    replay(format, history).map_err(|err| {
        let id = id.get();
        match err {
            Contradiction::Engine(e) => tracing::error!(%e, id, "histórico gravado não fecha"),
            Contradiction::SidesChanged(m) => {
                tracing::error!(?m, id, "resultado gravado com outros lados");
            }
        }
        ApiError::internal()
    })
}

/// TDM com o draft: só nele a fase lê quem jogou cada partida enviada (no
/// resto, o pick é do lado).
fn needs_lineups(rules: &VetoRules) -> bool {
    rules.champion_phase && rules.teams()
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
    let format = *config(tournament, number)?;
    let veto_rules = tournament.setup.veto_rules();
    let entrants = teams::seeded(conn, tournament.id).await?;
    let parts = Parts {
        results: true,
        games: true,
        pairings: matches!(format, ef::StageFormat::Swiss(_)),
        veto: true,
        lineups: needs_lineups(&veto_rules),
        champions: veto_rules.champion_phase,
        ..Parts::default()
    };
    let db::StageRows {
        results,
        games,
        rounds,
        veto: veto_rows,
        lineups,
        champions,
        ..
    } = db::stage_rows(conn, &[tournament.id], number, parts)
        .await?
        .pop()
        .unwrap_or_default();
    let history = History {
        entrants: &entrants,
        results: &results,
        games: &games,
        rounds: &rounds,
    };
    let bracket = replayed(tournament.id, format, &history)?;
    let vetoes = vetoes(
        &bracket,
        &veto_rules,
        veto_rows,
        &entrants,
        &games,
        &lineups,
    )?;
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
        team_pick_seconds: tournament.setup.veto.team_pick_seconds,
        champions,
        map_pool: tournament.map_pool.clone(),
        vetoes,
        team_rules: tournament.setup.team_rules,
        lineups,
    })
}

/// A chave de um torneio iniciado sem o veto (`brackets`).
pub struct Settled {
    /// Os times da chave, na ordem dos seeds.
    pub entrants: Vec<StoredTeam>,
    pub bracket: StageBracket,
    /// Os mapas enviados (para quem confere as confirmações por tempo).
    pub games: Vec<StoredGame>,
}

/// A chave de cada torneio iniciado de `tournaments`, na ordem, sem o veto,
/// as escalações e os campeões: para dizer onde cada um terminou (a lista dos
/// torneios de quem vê). Duas idas ao banco para todos.
pub async fn brackets(
    conn: &mut PgConnection,
    tournaments: &[TournamentId],
) -> Result<Vec<Settled>, ApiError> {
    if tournaments.is_empty() {
        return Ok(Vec::new());
    }
    let teams = teams::seeded_in(conn, tournaments).await?;
    let parts = Parts {
        config: true,
        results: true,
        games: true,
        pairings: true,
        ..Parts::default()
    };
    let rows = db::stage_rows(conn, tournaments, FIRST_STAGE, parts).await?;
    tournaments
        .iter()
        .zip(teams)
        .zip(rows)
        .map(|((&id, entrants), rows)| {
            let format = rows.config.ok_or_else(ApiError::internal)?;
            let history = History {
                entrants: &entrants,
                results: &rows.results,
                games: &rows.games,
                rounds: &rows.rounds,
            };
            let bracket = replayed(id, format, &history)?;
            Ok(Settled {
                entrants,
                bracket,
                games: rows.games,
            })
        })
        .collect()
}

impl Stage {
    /// A fase depois de uma ação que, na transação, só gravou `wrote`,
    /// relendo só isso numa ida (o mesmo que `load`, com menos idas ao
    /// banco): os times, as séries decididas, os pareamentos e os campeões
    /// não mudaram. A chave e o veto saem do replay de novo. Quem grava outra
    /// coisa usa `load`.
    pub async fn refresh(
        self,
        conn: &mut PgConnection,
        tournament: &StoredTournament,
        wrote: Wrote,
    ) -> Result<Self, ApiError> {
        let parts = match wrote {
            Wrote::Veto => Parts {
                veto: true,
                ..Parts::default()
            },
            Wrote::Games => Parts {
                games: true,
                lineups: needs_lineups(&self.veto_rules),
                ..Parts::default()
            },
        };
        let fresh = db::stage_rows(conn, &[tournament.id], self.number, parts)
            .await?
            .pop()
            .unwrap_or_default();
        let (games, bracket, lineups, rows) = match wrote {
            Wrote::Veto => (self.games, self.bracket, self.lineups, fresh.veto),
            Wrote::Games => {
                let games = fresh.games;
                let lineups = fresh.lineups;
                let history = History {
                    entrants: &self.entrants,
                    results: &self.results,
                    games: &games,
                    rounds: &self.rounds,
                };
                let bracket = replayed(tournament.id, self.format, &history)?;
                let rows = self
                    .vetoes
                    .into_iter()
                    .flat_map(|v| {
                        let (match_id, sides) = (v.match_id, v.sides);
                        v.log.rows.into_iter().map(move |row| MatchVetoRow {
                            match_id,
                            sides,
                            row,
                        })
                    })
                    .collect();
                (games, bracket, lineups, rows)
            }
        };
        let vetoes = vetoes(
            &bracket,
            &self.veto_rules,
            rows,
            &self.entrants,
            &games,
            &lineups,
        )?;
        Ok(Self {
            games,
            bracket,
            vetoes,
            lineups,
            ..self
        })
    }
}

/// O veto de cada série refeito das linhas `rows` (por série, na ordem), com
/// o elenco de cada lado (TDM) dos times `entrants`, a escalação de cada
/// partida enviada (`games`; no TDM, quem jogou, de `lineups`) e se a série
/// acabou: a janela simultânea não reabre numa partida enviada.
fn vetoes(
    bracket: &StageBracket,
    rules: &VetoRules,
    rows: Vec<MatchVetoRow>,
    entrants: &[StoredTeam],
    games: &[StoredGame],
    lineups: &[(MatchId, i16, Side, i64)],
) -> Result<Vec<MatchVeto>, ApiError> {
    let mut by_match: Vec<(MatchId, (ParticipantId, ParticipantId), Vec<_>)> = Vec::new();
    for MatchVetoRow {
        match_id,
        sides,
        row,
    } in rows
    {
        match by_match.last_mut() {
            Some((id, _, group)) if *id == match_id => group.push(row),
            _ => by_match.push((match_id, sides, vec![row])),
        }
    }
    by_match
        .into_iter()
        .map(|(id, sides, rows)| {
            let m = bracket.get(id).ok_or_else(ApiError::internal)?;
            let sent = games.iter().filter(|g| g.match_id == id).count();
            let lineups = (1..=sent).map(|n| lineup(lineups, id, n)).collect();
            let rosters = Rosters {
                players: [roster(entrants, sides.0), roster(entrants, sides.1)],
                lineups,
                finished: matches!(m.state, MatchState::Completed { .. }),
            };
            let log =
                VetoLog::replay(rules.clone(), m.best_of, rosters, rows).inspect_err(|_| {
                    tracing::error!(?id, "o veto desta série não fecha");
                })?;
            Ok(MatchVeto {
                match_id: id,
                sides,
                log,
            })
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
    // Os times que entram estão completos (`teams::finalize_seeding`).
    let players = seeded.len() * tournament.setup.team_size() as usize;
    tournament.setup.check_entrants(pool, players)?;
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
/// deixa partida por jogar. Devolve o status que vale. Sem mudança (o
/// normal), não grava: o status de `tournament` é o lido com o torneio
/// travado, o mesmo que `changes::settled` compara.
pub async fn sync_status(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    stage: &Stage,
) -> Result<TournamentStatus, ApiError> {
    let finished = stage.bracket.is_finished();
    let status = if finished {
        TournamentStatus::Finished
    } else {
        TournamentStatus::InProgress
    };
    if status == tournament.status {
        return Ok(status);
    }
    Ok(tournaments::sync_finished(conn, tournament.id, finished).await?)
}
