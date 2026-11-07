//! O que a fase grava, numa ida só ao banco: as séries decididas, os mapas,
//! os pareamentos do suíço, os eventos de veto, as escalações, os campeões
//! e as decisões dos empates do fim,
//! cada parte agregada em JSON e lida só quando pedida (`Parts`). Serve a
//! `stage::load`, a `Stage::refresh` e às colocações dos torneios encerrados
//! (`mine::final_places`, vários torneios de uma vez). Os times da chave ficam
//! com `teams` (`seeded`, `seeded_in`).

use chrono::{DateTime, Utc};
use engine::{
    core::{ChampionId, MapId, MatchId, ParticipantId},
    formats::swiss::SwissRound,
    stages::StageSetup,
};
use serde::Deserialize;
use sqlx::{PgConnection, types::Json};

use super::pairings::{PairingRow, rounds};
use super::ties::TieRow;
use crate::{
    matches::{
        games::{GameRow, StoredGame},
        key,
        result::{ResultRow, Side, StoredResult, unknown},
        veto::{Champion, Kind, MatchVetoRow, VetoRow},
    },
    shared::ApiError,
    tournaments::{StageProgress, StageRow, TournamentId},
};

/// O que ler da fase. O que fica de fora vem vazio.
#[derive(Clone, Copy, Debug, Default)]
pub struct Parts {
    /// A configuração da fase (`tournament_stages`), para quem não tem o
    /// torneio lido.
    pub config: bool,
    pub results: bool,
    pub games: bool,
    pub pairings: bool,
    pub veto: bool,
    /// TDM com o draft: quem jogou cada partida enviada.
    pub lineups: bool,
    pub champions: bool,
    /// Fases de tabela com a organização decidindo o empate do fim: as
    /// decisões e se a fase espera por ela.
    pub ties: bool,
}

/// A fase de um torneio como está gravada.
#[derive(Default)]
pub struct StageRows {
    /// O id do torneio e a posição da fase.
    pub tournament: i64,
    pub stage: i16,
    /// Quando a fase terminou, como está gravado agora (o torneio lido antes
    /// pode estar atrás, se a fase terminou ou reabriu na mesma transação).
    pub finished_at: Option<DateTime<Utc>>,
    /// A configuração e o andamento da fase.
    pub config: Option<(StageSetup, StageProgress)>,
    /// Na ordem do replay.
    pub results: Vec<StoredResult>,
    /// Por partida e na ordem.
    pub games: Vec<StoredGame>,
    /// Suíço: os pareamentos, da rodada 1 em diante.
    pub rounds: Vec<SwissRound>,
    /// Os eventos de veto, por partida e na ordem, com o apelido de quem agiu.
    pub veto: Vec<MatchVetoRow>,
    /// `(série, partida, lado, jogador)`, na ordem.
    pub lineups: Vec<(MatchId, i16, Side, i64)>,
    /// Todos os campeões (os inativos aparecem em vetos antigos), na ordem
    /// dos ids: a ordem das opções e do sorteio do tempo esgotado.
    pub champions: Vec<Champion>,
    /// As decisões dos empates do fim, na ordem em que foram gravadas.
    pub ties: Vec<TieRow>,
    /// Desde quando a fase espera a organização decidir um empate.
    pub ties_pending_since: Option<DateTime<Utc>>,
}

/// Que fase ler de um torneio em `stage_rows`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Which {
    /// A fase de número dado.
    Stage(i16),
    /// Todas as que começaram (a 1 sempre), na ordem.
    Started,
}

impl Which {
    const fn number(self) -> i16 {
        match self {
            Self::Stage(n) => n,
            Self::Started => 0,
        }
    }
}

/// As fases `which` de cada torneio de `at`, na mesma ordem (com
/// `Which::Started`, as do torneio em sequência, da 1 em diante), com as
/// partes de `parts`. Uma query, qualquer que seja o número de torneios.
///
/// Os ids e as partes entram por `(select $n)`: assim o plano genérico do
/// statement preparado custa o mesmo que o feito para os valores, e o
/// Postgres passa a reusá-lo (sem isso, replanejava a query a cada chamada:
/// 0,48 ms contra 0,01 ms, medido no PG 18 local).
pub async fn stage_rows(
    conn: &mut PgConnection,
    at: &[(TournamentId, Which)],
    parts: Parts,
) -> Result<Vec<StageRows>, ApiError> {
    let ids: Vec<i64> = at.iter().map(|(t, _)| t.get()).collect();
    let stages: Vec<i16> = at.iter().map(|(_, w)| w.number()).collect();
    let rows = sqlx::query!(
        r#"select q.id as "id!", st.position as "stage!", st.finished_at,
                  case when (select $3::bool) then (
                      select to_json(s) from (
                          select format, best_of, finals_best_of, bracket, grand_final_reset,
                                 rounds, tiebreakers, final_tie, advance, check_in, started_at,
                                 check_in_opened_at, finished_at
                          from tournament_stages
                          where tournament_id = q.id and position = st.position) s)
                  end as "config: Json<StageRow>",
                  case when (select $4::bool) then (
                      select json_agg(r order by r.position) from (
                          select id, match_key, side_a, side_b, winner, a_wins, b_wins,
                                 confirmed_via, confirmed_at, position
                          from match_results where tournament_id = q.id and stage = st.position) r)
                  end as "results: Json<Vec<ResultRow>>",
                  case when (select $5::bool) then (
                      select json_agg(g order by g.match_key, g.number) from (
                          select g.id, g.match_key, g.number, g.side_a, g.side_b, g.winner,
                                 g.map_id, m.slug as map_slug, m.name as map_name, g.a_frags,
                                 g.b_frags, g.status, g.reported_side, g.reported_by,
                                 g.reported_at, g.confirm_by, g.contested_by, g.contested_at,
                                 g.contest_comment, g.confirmed_via, g.confirmed_at
                          from match_games g
                          left join maps m on m.id = g.map_id
                          where g.tournament_id = q.id and g.stage = st.position) g)
                  end as "games: Json<Vec<GameRow>>",
                  case when (select $6::bool) then (
                      select json_agg(p order by p.round, p.number) from (
                          select round, number, side_a, side_b from stage_pairings
                          where tournament_id = q.id and stage = st.position) p)
                  end as "pairings: Json<Vec<PairingRow>>",
                  case when (select $7::bool) then (
                      select json_agg(e order by e.match_key, e.id) from (
                          select e.id, e.match_key, e.side_a, e.side_b, e.game, e.kind, e.side,
                                 e.map_id, e.played_map_id, e.champion_id, e.player_id,
                                 e.pick_game, e.gave_way, e.seed, e.timeout, e.actor,
                                 coalesce(m.nickname, u.display_name) as actor_name,
                                 pm.nickname as player_name, e.created_at
                          from match_veto_events e
                          left join users u on u.id = e.actor
                          left join tenant_members m
                                 on m.tenant_id = e.tenant_id and m.user_id = e.actor
                          left join tenant_members pm
                                 on pm.tenant_id = e.tenant_id and pm.user_id = e.player_id
                          where e.tournament_id = q.id and e.stage = st.position) e)
                  end as "veto: Json<Vec<VetoEventRow>>",
                  case when (select $8::bool) then (
                      select json_agg(l order by l.match_key, l.number, l.side, l.user_id) from (
                          select g.match_key, g.number, gp.side, gp.user_id
                          from match_game_players gp
                          join match_games g on g.id = gp.game_id
                          where g.tournament_id = q.id and g.stage = st.position) l)
                  end as "lineups: Json<Vec<PlayedRow>>",
                  case when (select $9::bool) then (
                      select json_agg(c order by c.id) from (
                          select id, slug, name, active from champions) c)
                  end as "champions: Json<Vec<ChampionRow>>",
                  case when (select $10::bool) then (
                      select json_agg(d order by d.id) from (
                          select d.id, d.reason,
                                 (select array_agg(x.team_id order by x.position)
                                  from stage_tie_decision_teams x
                                  where x.decision_id = d.id) as teams
                          from stage_tie_decisions d
                          where d.tournament_id = q.id and d.stage = st.position) d)
                  end as "ties: Json<Vec<TieRow>>",
                  case when (select $10::bool) then (
                      select since from stage_pending_ties
                      where tournament_id = q.id and stage = st.position)
                  end as ties_pending_since
           from unnest((select $1::bigint[]), (select $2::smallint[]))
                    with ordinality as q(id, stage, n)
           join tournament_stages st
             on st.tournament_id = q.id
            and (st.position = q.stage
                 or (q.stage = 0 and (st.started_at is not null or st.position = 1)))
           order by q.n, st.position"#,
        &ids,
        &stages,
        parts.config,
        parts.results,
        parts.games,
        parts.pairings,
        parts.veto,
        parts.lineups,
        parts.champions,
        parts.ties
    )
    .fetch_all(conn)
    .await?;
    rows.into_iter()
        .map(|r| {
            Ok(StageRows {
                tournament: r.id,
                stage: r.stage,
                finished_at: r.finished_at,
                config: r
                    .config
                    .map(|c| Ok::<_, ApiError>((c.0.read_setup()?, c.0.progress())))
                    .transpose()?,
                results: list(r.results)
                    .into_iter()
                    .map(StoredResult::try_from)
                    .collect::<Result<_, _>>()?,
                games: list(r.games)
                    .into_iter()
                    .map(StoredGame::try_from)
                    .collect::<Result<_, _>>()?,
                rounds: rounds(list(r.pairings))?,
                veto: list(r.veto)
                    .into_iter()
                    .map(MatchVetoRow::try_from)
                    .collect::<Result<_, _>>()?,
                lineups: list(r.lineups)
                    .into_iter()
                    .map(PlayedRow::read)
                    .collect::<Result<_, _>>()?,
                champions: list(r.champions)
                    .into_iter()
                    .map(ChampionRow::read)
                    .collect(),
                ties: list(r.ties),
                ties_pending_since: r.ties_pending_since,
            })
        })
        .collect()
}

/// Uma parte que não foi pedida (ou que não tem linhas) vem nula: vazia.
fn list<T>(json: Option<Json<Vec<T>>>) -> Vec<T> {
    json.map(|j| j.0).unwrap_or_default()
}

/// Um evento de veto como vem do JSON de `stage_rows`.
#[derive(Deserialize, Debug)]
struct VetoEventRow {
    id: i64,
    match_key: String,
    side_a: i64,
    side_b: i64,
    game: Option<i16>,
    kind: String,
    side: Option<String>,
    map_id: Option<i64>,
    played_map_id: Option<i64>,
    champion_id: Option<i64>,
    player_id: Option<i64>,
    pick_game: Option<i16>,
    gave_way: bool,
    seed: Option<i64>,
    timeout: bool,
    actor: Option<i64>,
    actor_name: Option<String>,
    player_name: Option<String>,
    created_at: DateTime<Utc>,
}

impl TryFrom<VetoEventRow> for MatchVetoRow {
    type Error = ApiError;

    fn try_from(r: VetoEventRow) -> Result<Self, ApiError> {
        Ok(Self {
            match_id: key::parse(&r.match_key).ok_or_else(|| unknown("match_key", &r.match_key))?,
            sides: (ParticipantId(r.side_a), ParticipantId(r.side_b)),
            row: VetoRow {
                id: r.id,
                game: r.game,
                kind: Kind::from_db(&r.kind)?,
                side: r.side.as_deref().map(Side::from_db).transpose()?,
                map: r.map_id.map(MapId),
                played_map: r.played_map_id.map(MapId),
                champion: r.champion_id.map(ChampionId),
                player: r.player_id,
                player_name: r.player_name,
                pick_game: r.pick_game,
                gave_way: r.gave_way,
                seed: r.seed,
                timeout: r.timeout,
                actor: r.actor,
                actor_name: r.actor_name,
                created_at: r.created_at,
            },
        })
    }
}

/// Quem jogou uma partida enviada por um lado (`match_game_players`).
#[derive(Deserialize, Debug)]
struct PlayedRow {
    match_key: String,
    number: i16,
    side: String,
    user_id: i64,
}

impl PlayedRow {
    fn read(self) -> Result<(MatchId, i16, Side, i64), ApiError> {
        let id =
            key::parse(&self.match_key).ok_or_else(|| unknown("match_key", &self.match_key))?;
        Ok((id, self.number, Side::from_db(&self.side)?, self.user_id))
    }
}

#[derive(Deserialize, Debug)]
struct ChampionRow {
    id: i64,
    slug: String,
    name: String,
    active: bool,
}

impl ChampionRow {
    fn read(self) -> Champion {
        Champion {
            id: ChampionId(self.id),
            slug: self.slug,
            name: self.name,
            active: self.active,
        }
    }
}
