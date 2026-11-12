//! Uma fase começada: configuração (`tournament_stages`), times (pelo seed
//! da fase: o de `teams` na 1, o de `stage_teams` nas outras), séries
//! decididas e, no suíço, os pareamentos gravados (`rounds`) → fase de
//! qualquer formato (`StageBracket`), sempre por `replay`. Nada do estado do
//! `engine` é gravado. Os mapas das séries e o veto de cada partida (também
//! refeito por replay) vêm junto. Duas idas ao banco: os times (`teams`) e o
//! resto (`db::stage_rows`, com os campeões quando há draft e, com a
//! organização decidindo o empate do fim, as decisões dela, `ties`). As
//! partidas de uma fase carregam a fase delas (`load` com o número); o
//! resto, a fase atual (a última que começou, `load_current`). Para a
//! colocação nos torneios encerrados, `brackets` refaz só a chave de todas as
//! fases de vários torneios, com as mesmas duas idas.
//!
//! A fase 1 nasce no início do torneio (`start`); as outras, pelo botão da
//! organização (`start_stage`, com os seeds já gravados), e desfazer o início
//! de uma delas a descarta (`discard_stage`). O fim da última encerra o
//! torneio; o das outras só fica marcado (`sync_status`).

use chrono::{DateTime, Utc};
use engine::{
    core::{EngineError, MatchId, MatchState, ParticipantId, PlayerId, SeriesFrags, StageReport},
    formats::{self as ef, StageBracket, StageHistory, swiss::SwissRound},
    stages::StageSetup,
    veto::{Rosters, VetoRules},
};
use sqlx::PgConnection;

use super::{
    changes,
    db::{self, Parts, Which},
    games::StoredGame,
    result::{Side, StoredResult},
    rounds,
    ties::{self, StoredTie},
    veto::{Champion, MatchVeto, MatchVetoRow, VetoLog},
};
use crate::{
    shared::ApiError,
    teams::{self, StoredTeam},
    tenants,
    tournaments::{
        self, PoolMap, StageProgress, StoredTournament, TeamRules, TournamentId, TournamentStatus,
    },
};

/// A fase que nasce com o torneio.
pub const FIRST_STAGE: i16 = 1;

pub struct Stage {
    pub number: i16,
    pub format: ef::StageFormat,
    /// A última fase do torneio: o fim dela encerra o torneio.
    pub last: bool,
    /// Quando a fase começou: a 1, o início do torneio (`None` em torneio
    /// antigo, sem ele); as outras, o botão da organização.
    pub started_at: Option<DateTime<Utc>>,
    /// Quando terminou, como está gravado (`sync_status` acerta).
    pub finished_at: Option<DateTime<Utc>>,
    /// A fase seguinte já começou: quem entrou nela e a ordem não podem mais
    /// mudar (`settle` confere), e a contestação tardia desta fecha.
    pub next_started: bool,
    /// Os times da chave, na ordem dos seeds da fase (com o seed dela).
    pub entrants: Vec<StoredTeam>,
    /// Os times que jogaram o torneio (os da fase 1, pelo seed do torneio),
    /// com as linhas de cada fase: para quem entra nas fases seguintes.
    pub teams: Vec<StoredTeam>,
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
    /// Fases de tabela com a organização decidindo o empate do fim: as
    /// decisões gravadas, na ordem; vazio nos outros casos.
    pub ties: Vec<StoredTie>,
    /// Desde quando a fase espera a organização decidir um empate (o que está
    /// gravado: `ties::sync` acerta depois de cada mudança).
    pub ties_pending_since: Option<DateTime<Utc>>,
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

    /// O apelido de `user_id` entre os jogadores dos times do torneio; `None`
    /// se ele não joga (a organização) ou a conta foi excluída.
    pub fn nickname(&self, user_id: i64) -> Option<&str> {
        self.teams
            .iter()
            .flat_map(|t| &t.members)
            .find(|m| m.user_id == user_id)
            .map(|m| m.nickname.as_str())
            .filter(|n| !n.is_empty())
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
            ties: &self.ties,
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
    /// As decisões dos empates do fim.
    pub ties: &'a [StoredTie],
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
        tie_orders: history.ties.iter().map(StoredTie::order).collect(),
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
fn config(tournament: &StoredTournament, number: i16) -> Result<ef::StageFormat, ApiError> {
    tournament
        .stage(number)
        .map(|(s, _)| s.format)
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

/// Fase de tabela com a organização decidindo o empate do fim: só nela a fase
/// lê as decisões e a marca de espera.
fn organizer_ties(format: &ef::StageFormat) -> bool {
    let final_tie = match format {
        ef::StageFormat::RoundRobin(c) => c.final_tie,
        ef::StageFormat::Swiss(c) => c.final_tie,
        _ => return false,
    };
    final_tie == ef::table::FinalTie::Organizer
}

/// TDM com o draft: só nele a fase lê quem jogou cada partida enviada (no
/// resto, o pick é do lado).
fn needs_lineups(rules: &VetoRules) -> bool {
    rules.champion_phase && rules.teams()
}

/// A fase atual (a última que começou), refeita. 404 `bracket.not_found`
/// antes de o torneio começar.
pub async fn load_current(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
) -> Result<Stage, ApiError> {
    load(conn, tournament, tournament.current_stage()).await
}

/// A fase `number`, refeita. 404 `bracket.not_found` antes de o torneio
/// começar e `stage.not_started` com a fase por começar (ou que não existe).
pub async fn load(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    number: i16,
) -> Result<Stage, ApiError> {
    if !matches!(
        tournament.status,
        TournamentStatus::InProgress | TournamentStatus::Finished
    ) {
        return Err(ApiError::not_found("bracket.not_found"));
    }
    if !tournament.stage_started(number) {
        return Err(ApiError::not_found("stage.not_started"));
    }
    let format = config(tournament, number)?;
    let veto_rules = tournament.setup.stage_veto_rules(number);
    let teams = teams::seeded(conn, tournament.id).await?;
    let entrants = teams::stage_entrants(&teams, number);
    let parts = Parts {
        results: true,
        games: true,
        pairings: matches!(format, ef::StageFormat::Swiss(_)),
        veto: true,
        lineups: needs_lineups(&veto_rules),
        champions: veto_rules.champion_phase,
        ties: organizer_ties(&format),
        ..Parts::default()
    };
    let db::StageRows {
        results,
        games,
        rounds,
        veto: veto_rows,
        lineups,
        champions,
        ties,
        ties_pending_since,
        finished_at,
        ..
    } = db::stage_rows(conn, &[(tournament.id, Which::Stage(number))], parts)
        .await?
        .pop()
        .unwrap_or_default();
    let ties = ties::stored(ties);
    let history = History {
        entrants: &entrants,
        results: &results,
        games: &games,
        rounds: &rounds,
        ties: &ties,
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
    let progress = tournament.stage(number).map(|(_, p)| p).unwrap_or_default();
    Ok(Stage {
        number,
        format,
        last: number == tournament.stage_count(),
        started_at: if number == FIRST_STAGE {
            tournament.started_at
        } else {
            progress.started_at
        },
        finished_at,
        next_started: tournament.stage_started(number + 1),
        entrants,
        teams,
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
        ties,
        ties_pending_since,
        lineups,
    })
}

/// A chave de uma fase sem o veto (`brackets`, `chain`).
pub struct Settled {
    pub number: i16,
    /// A configuração (o corte e o check-in) e o andamento da fase.
    pub setup: StageSetup,
    pub progress: StageProgress,
    /// Os times da fase, na ordem dos seeds dela.
    pub entrants: Vec<StoredTeam>,
    pub bracket: StageBracket,
    /// Os mapas enviados (para quem confere as confirmações por tempo).
    pub games: Vec<StoredGame>,
}

/// As fases começadas de um torneio, em sequência, sem o veto.
pub struct Chain {
    /// Os times que jogaram o torneio (os da fase 1), com as linhas de cada
    /// fase.
    pub teams: Vec<StoredTeam>,
    /// Da fase 1 à atual.
    pub stages: Vec<Settled>,
    /// A atual é a última fase do torneio (a colocação dela é a final).
    pub last: bool,
}

/// As partes que a chave sem o veto lê.
const SETTLED: Parts = Parts {
    config: true,
    results: true,
    games: true,
    pairings: true,
    veto: false,
    lineups: false,
    champions: false,
    ties: true,
};

/// A chave de cada fase começada de cada torneio encerrado de `tournaments`,
/// na ordem, sem o veto, as escalações e os campeões: para dizer onde cada
/// um terminou (a lista dos torneios de quem vê). Duas idas ao banco para
/// todos.
pub async fn brackets(
    conn: &mut PgConnection,
    tournaments: &[TournamentId],
) -> Result<Vec<Chain>, ApiError> {
    if tournaments.is_empty() {
        return Ok(Vec::new());
    }
    let teams = teams::seeded_in(conn, tournaments).await?;
    let at: Vec<_> = tournaments.iter().map(|&t| (t, Which::Started)).collect();
    let mut rows = db::stage_rows(conn, &at, SETTLED)
        .await?
        .into_iter()
        .peekable();
    tournaments
        .iter()
        .zip(teams)
        .map(|(&id, teams)| {
            let mut stages = Vec::new();
            while let Some(r) = rows.next_if(|r| r.tournament == id.get()) {
                stages.push(settled(id, &teams, r)?);
            }
            // Encerrado: a fase atual é a última.
            Ok(Chain {
                teams,
                stages,
                last: true,
            })
        })
        .collect()
}

/// As fases começadas do torneio, da 1 à atual, reaproveitando `have`, já
/// lida com o veto (a que quem chama vê). Com uma fase começada só, sem ida
/// ao banco.
pub async fn chain(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    have: &Stage,
) -> Result<Chain, ApiError> {
    let of = |s: &Stage| -> Result<Settled, ApiError> {
        let (setup, progress) = tournament.stage(s.number).ok_or_else(ApiError::internal)?;
        Ok(Settled {
            number: s.number,
            setup: *setup,
            progress,
            entrants: s.entrants.clone(),
            bracket: s.bracket.clone(),
            games: s.games.clone(),
        })
    };
    let current = tournament.current_stage();
    let last = current == tournament.stage_count();
    if current == FIRST_STAGE {
        return Ok(Chain {
            teams: have.teams.clone(),
            stages: vec![of(have)?],
            last,
        });
    }
    // As outras fases começadas, numa ida; a lida já vem de `have`.
    let others: Vec<_> = (FIRST_STAGE..=current)
        .filter(|&n| n != have.number)
        .map(|n| (tournament.id, Which::Stage(n)))
        .collect();
    let mut rows = db::stage_rows(conn, &others, SETTLED).await?.into_iter();
    let stages = (FIRST_STAGE..=current)
        .map(|n| {
            if n == have.number {
                return of(have);
            }
            let r = rows.next().ok_or_else(ApiError::internal)?;
            settled(tournament.id, &have.teams, r)
        })
        .collect::<Result<_, ApiError>>()?;
    Ok(Chain {
        teams: have.teams.clone(),
        stages,
        last,
    })
}

/// A fase das linhas `rows`, com os times dela tirados de `teams`.
fn settled(
    id: TournamentId,
    teams: &[StoredTeam],
    rows: db::StageRows,
) -> Result<Settled, ApiError> {
    let (setup, progress) = rows.config.ok_or_else(ApiError::internal)?;
    let entrants = teams::stage_entrants(teams, rows.stage);
    let ties = ties::stored(rows.ties);
    let history = History {
        entrants: &entrants,
        results: &rows.results,
        games: &rows.games,
        rounds: &rows.rounds,
        ties: &ties,
    };
    let bracket = replayed(id, setup.format, &history)?;
    Ok(Settled {
        number: rows.stage,
        setup,
        progress,
        entrants,
        bracket,
        games: rows.games,
    })
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
        let at = [(tournament.id, Which::Stage(self.number))];
        let fresh = db::stage_rows(conn, &at, parts)
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
                    ties: &self.ties,
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
    let format = config(tournament, FIRST_STAGE)?;
    let at = (tournament.tenant, tournament.id, FIRST_STAGE);
    let bracket = rounds::first(conn, at, format, seeded).await?;
    tournaments::mark_stage_started(conn, tournament.id, FIRST_STAGE).await?;
    let entrants = teams::seeded(conn, tournament.id).await?;
    changes::started(conn, tournament, FIRST_STAGE, &bracket, &entrants).await
}

/// Gera a fase `number` (2 em diante), no torneio travado, com os seeds dela
/// já gravados (`seeded`, na ordem) e o início marcado: confere a
/// configuração dela com os times de verdade e o pool de campeões de agora,
/// monta a fase (no suíço, grava o pareamento da primeira rodada) e avisa
/// os lados das partidas que já nascem prontas e o Discord. Erros: os de
/// `Setup::check_stage` e os de montar a fase.
pub async fn start_stage(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    number: i16,
    seeded: &[ParticipantId],
) -> Result<(), ApiError> {
    let pool = tournaments::champion_pool(&mut *conn).await?;
    tournament.setup.check_stage(pool, number, seeded.len())?;
    let format = config(tournament, number)?;
    let at = (tournament.tenant, tournament.id, number);
    let bracket = rounds::first(conn, at, format, seeded).await?;
    let teams = teams::seeded(conn, tournament.id).await?;
    let entrants = teams::stage_entrants(&teams, number);
    changes::started(conn, tournament, number, &bracket, &entrants).await
}

/// Descarta todas as fases ao desfazer o início do torneio: apaga séries,
/// mapas, histórico, pareamentos, o andamento de cada fase e os times das
/// fases seguintes (seeds e check-ins). Devolve quantas partidas tinham
/// resultado ou mapa.
pub async fn discard(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
) -> Result<u64, ApiError> {
    let touched = db::discard(conn, tournament.id).await?;
    tournaments::reset_stages(conn, tournament.id).await?;
    teams::clear_stages(conn, tournament.id).await?;
    Ok(touched)
}

/// Desfaz o início da fase `number` (2 em diante), a atual: só sem nada
/// feito nela (nenhum evento de veto ou de partida, mapa, resultado ou
/// screenshot; senão, 409
/// `stage.has_activity`). Apaga os pareamentos, o início e os seeds dela; os
/// check-ins ficam.
pub async fn discard_stage(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    number: i16,
) -> Result<(), ApiError> {
    if db::stage_activity(conn, tournament.id, number).await? {
        return Err(ApiError::conflict("stage.has_activity"));
    }
    db::discard_pairings(conn, tournament.id, number).await?;
    tournaments::unmark_stage_started(conn, tournament.id, number).await?;
    teams::clear_stage_seeds(conn, tournament.id, number).await?;
    Ok(())
}

/// Marca o fim da fase quando ela termina (e o tira quando uma correção a
/// reabre); a última fase encerra o torneio e o reabre do mesmo jeito. A
/// fase que termina e tem seguinte não encerra nada nem avança sozinha: a
/// organização inicia a seguinte, e a home dela recarrega. Devolve o status
/// que vale. Sem mudança (o normal), não grava: o status de `tournament` é o
/// lido com o torneio travado, o mesmo que `changes::settled` compara.
pub async fn sync_status(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    stage: &mut Stage,
) -> Result<TournamentStatus, ApiError> {
    let finished = stage.bracket.is_finished();
    // `finished_at` é o lido com a fase (`load`); a gravação só muda o que
    // ainda está diferente no banco, e a fase fica com o que valeu.
    if finished != stage.finished_at.is_some()
        && let Some(at) =
            tournaments::mark_stage_finished(conn, tournament.id, stage.number, finished).await?
    {
        stage.finished_at = at;
        if !stage.last {
            tournaments::changed(conn, tournament.id).await?;
            tenants::organizing_changed(conn, tournament.tenant).await?;
        }
    }
    if !stage.last {
        return Ok(tournament.status);
    }
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
