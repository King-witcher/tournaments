use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{
    games::GameStatus,
    key::MatchKey,
    result::{Confirmation, Side},
    veto::{VetoStatus, VetoView},
};
use crate::{
    teams::{Team, TeamMember},
    tournaments::{StageFormat, TournamentStatus},
};

/// `?stage=` de `GET .../bracket`.
#[derive(Deserialize)]
pub struct BracketQuery {
    /// A fase (de 1 em diante); ausente: a atual (a última que começou).
    pub stage: Option<i16>,
}

/// A chave de uma fase, refeita dos resultados confirmados.
#[derive(Serialize, ToSchema)]
pub struct TournamentBracket {
    /// A fase desta chave, de 1 em diante.
    pub stage: u16,
    pub format: StageFormat,
    /// Status do torneio depois das confirmações por tempo.
    pub status: TournamentStatus,
    /// Os times da chave, na ordem dos seeds.
    pub entrants: Vec<Team>,
    /// Todas as partidas, por parte da chave, rodada e índice. No suíço, só
    /// as das rodadas já pareadas.
    pub matches: Vec<BracketMatch>,
    /// Colocações já definidas. Na eliminação, os eliminados e, no fim,
    /// campeão e vice; empatados dividem a faixa (`rank` 5 com `tied` 4 é
    /// 5º–8º). No round robin e no suíço, todos, só no fim, sem empate. Na
    /// disputa de posições, as dos pares decididos e a do sem par, sem empate.
    /// Com um empate do fim esperando a organização, os do grupo ficam de fora
    /// (`pending_ties`). São as colocações desta fase.
    pub standings: Vec<Standing>,
    /// A classificação final do torneio, com as posições já conhecidas: os da
    /// última fase pela colocação dela (só quando a fase atual é a última);
    /// depois, de trás para a frente, os cortados de cada fase e, por último
    /// dela, os que não confirmaram presença ou desistiram. Com uma fase só, é
    /// `standings`.
    pub standings_final: Vec<Standing>,
    /// Os times de `standings_final` que não estão em `entrants` (os que
    /// ficaram em fases anteriores).
    pub other_teams: Vec<Team>,
    /// Round robin e suíço: a tabela de agora, em ordem de posição (a final
    /// quando a fase termina); `null` na eliminação e na disputa de posições.
    pub table: Option<Vec<TableRow>>,
    /// Round robin e suíço com a organização decidindo o empate do fim: os
    /// grupos que, com todas as séries decididas, esperam a decisão dela. A
    /// fase (e o torneio) só termina sem nenhum. Vazio nos outros casos.
    pub pending_ties: Vec<PendingTieGroup>,
    /// Round robin e suíço: o número de rodadas da fase (no suíço, as que vão
    /// ser jogadas, não só as já pareadas); `null` na eliminação e na disputa
    /// de posições.
    pub total_rounds: Option<u16>,
    /// Janela de contestação de cada mapa, em minutos.
    pub contest_minutes: i32,
    /// Partidas com algum mapa enviado ou resultado, que desfazer o início
    /// apaga.
    pub results: i64,
}

/// Parte da chave.
#[derive(Serialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BracketPart {
    /// Chave de cima (a única, na eliminação simples).
    Winners,
    /// Chave de baixo da eliminação dupla.
    Losers,
    /// Grande final (rodada 1) e reset (rodada 2).
    GrandFinal,
    /// Rodada do round robin.
    RoundRobin,
    /// Rodada do suíço.
    Swiss,
    /// Os pares da disputa de posições (rodada 1; o par k vale as posições
    /// 2k − 1 e 2k).
    Placement,
}

/// Uma linha da tabela do round robin ou do suíço.
#[derive(Serialize, ToSchema)]
pub struct TableRow {
    /// O time (um dos `entrants` da chave).
    pub team_id: i64,
    /// De 1 a N, sem empate, menos num empate que espera a organização
    /// (`tie_pending`): os do grupo dividem a melhor posição dele.
    pub position: u16,
    /// Está num empate do fim que espera a decisão da organização; a ordem
    /// dentro do grupo é a provisória (a da chave).
    pub tie_pending: bool,
    /// Num grupo que a organização ordenou (`ahead_by` `organizer`): o motivo
    /// dela, se deu.
    pub tie_reason: Option<String>,
    /// Séries vencidas (W.O. inclusive) e, no suíço, os byes. É o primeiro
    /// critério do suíço.
    pub points: u16,
    /// Vitórias menos derrotas (W.O. inclusive; empate vale 0). É o primeiro
    /// critério do round robin.
    pub series_diff: i32,
    /// Séries decididas (W.O. e empate inclusive; bye e folga não).
    pub played: u16,
    pub wins: u16,
    /// Séries empatadas: só nas séries pares.
    pub draws: u16,
    pub losses: u16,
    /// Byes do suíço (cada um vale um ponto). No round robin, sempre 0: a
    /// folga não vale ponto.
    pub byes: u16,
    /// Mapas das séries; o W.O. conta como vitória sem perder mapa (2 x 0 na
    /// md2 e na md3, 3 x 0 na md4 e na md5), e a série empatada, metade para
    /// cada lado.
    pub maps_won: u16,
    pub maps_lost: u16,
    pub map_diff: i32,
    /// Frags das séries jogadas que vieram com frags (o W.O. não soma); podem
    /// ser negativos (suicídio e queda tiram frag).
    pub frags_for: i64,
    pub frags_against: i64,
    pub frag_diff: i64,
    /// Toda série jogada veio com frags: só assim o saldo de frags desempata.
    pub frags_complete: bool,
    /// Só no suíço com o Buchholz entre os desempates: a soma dos pontos dos
    /// adversários.
    pub buchholz: Option<u32>,
    /// Só com o menor rating entre os desempates: o rating do time congelado
    /// no início (média dos jogadores, com até 2 casas).
    pub rating: Option<f64>,
    /// O critério que pôs esta linha à frente da seguinte; `null` na última.
    pub ahead_by: Option<TableCriterion>,
}

/// Critério de ordem da tabela: o saldo de séries (round robin) ou os pontos
/// (suíço), depois os desempates da fase (`StageFormat`, `tiebreakers`), na
/// ordem dela, e, por último, o seed ou, no fim da fase, a decisão da
/// organização (`final_tie`).
#[derive(Serialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TableCriterion {
    SeriesDiff,
    Points,
    Buchholz,
    HeadToHead,
    MapDiff,
    FragDiff,
    LowerRating,
    Seed,
    /// A decisão da organização, no empate que sobrou depois de todos os
    /// critérios.
    Organizer,
}

/// Um empate do fim da fase que espera a organização.
#[derive(Serialize, ToSchema)]
pub struct PendingTieGroup {
    /// A melhor posição do grupo, que os empatados dividem.
    pub position: u16,
    /// Os times empatados, na ordem da chave.
    pub team_ids: Vec<i64>,
}

/// A ordem que a organização decide para um empate do fim da fase.
#[derive(Deserialize, ToSchema)]
pub struct TieDecision {
    /// Os times de um empate pendente (`pending_ties`), do melhor ao pior,
    /// todos e cada um uma vez. A ordem da chave também vale.
    pub teams: Vec<i64>,
    /// Motivo opcional, até 280 caracteres, que a classificação mostra.
    #[serde(default)]
    pub reason: Option<String>,
}

/// Estado da partida na chave (só séries decididas contam).
#[derive(Serialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MatchState {
    /// Algum lado ainda depende de outra partida.
    Pending,
    /// Os dois lados são conhecidos: dá para jogar (pode ter mapas enviados).
    Ready,
    /// Um lado vazio: o outro avança sem jogar.
    Bye,
    /// Série decidida.
    Completed,
    /// Reset da grande final que não precisou acontecer.
    NotNeeded,
}

/// O que a partida (ou um mapa dela) espera de quem vê.
#[derive(Serialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViewerTask {
    /// Enviar o próximo mapa (joga a partida e a série está aberta).
    Report,
    /// Confirmar ou contestar um mapa que o adversário enviou.
    Respond,
    /// Enviou e espera o adversário (ou o fim da janela).
    AwaitOpponent,
    /// Mapa contestado: espera o capitão adversário de quem contestou ou o
    /// organizador.
    AwaitOrganizer,
    /// Organizer ou owner: resolver um mapa contestado.
    Resolve,
    /// Capitão do time adversário de quem contestou: pode aceitar a contestação.
    AcceptContest,
    /// Clicar em "Pronto" para o veto começar.
    Ready,
    /// Fazer o passo do veto que está na vez dele (na escolha simultânea do
    /// TDM, escolher o próprio campeão).
    Veto,
    /// O lado dele tem o que fazer, e a regra do torneio deixa com o capitão.
    AwaitCaptain,
    /// Ele já marcou "Pronto"; faltam colegas.
    AwaitTeam,
}

/// A próxima partida de quem vê num torneio em andamento.
#[derive(Serialize, ToSchema)]
pub struct NextMatch {
    #[schema(example = "wb-2-1")]
    pub key: String,
    /// `ready` (dá para jogar) ou `pending` (o adversário sai de outra
    /// partida).
    pub state: MatchState,
    /// Eliminação simples: o código da partida não tem a parte da chave.
    pub single: bool,
    /// O time adversário, quando já se sabe.
    pub opponent: Option<Team>,
    /// De que partida sai o adversário, enquanto não se sabe.
    pub opponent_from: Option<SideSource>,
    /// O que a partida espera de quem vê; `null` se nada.
    pub task: Option<ViewerTask>,
    /// Onde o veto da série pronta está; `null` sem veto no torneio ou fora da
    /// série pronta.
    pub veto: Option<VetoStatus>,
    /// O mapa (a arena) do próximo jogo a enviar na série, quando o veto, a
    /// votação ou o sorteio já o deu; `null` antes disso, sem veto de mapas e
    /// com a série decidida.
    pub next_map: Option<MapRef>,
    /// Round robin: a série ainda não começou e o adversário está jogando
    /// outra.
    pub opponent_playing: bool,
}

#[derive(Serialize, ToSchema)]
pub struct BracketMatch {
    /// Identificador na rota: `wb-1-3`, `lb-2-1`, `gf-1-1`, `rr-2-1`, `sw-3-2`, `pl-1-2`; da
    /// fase 2 em diante, com o prefixo da fase (`s2-pl-1-1`).
    #[schema(example = "wb-1-3")]
    pub key: String,
    pub part: BracketPart,
    pub round: u16,
    pub index: u16,
    pub best_of: u8,
    pub state: MatchState,
    pub a: MatchSide,
    pub b: MatchSide,
    /// A série decidida, se já está.
    pub result: Option<MatchResult>,
    /// Mapas confirmados vencidos por cada lado (o placar da série até agora).
    pub a_maps: u8,
    pub b_maps: u8,
    /// Frags somados da série decidida, de cada lado (os que entram na
    /// tabela); `null` antes da decisão, no W.O. e se algum mapa veio sem
    /// frags. Podem ser negativos.
    pub a_frags: Option<i32>,
    pub b_frags: Option<i32>,
    /// Mapas enviados esperando o adversário (ou o fim da janela).
    pub pending_maps: u8,
    /// Mapas contestados, esperando o organizador.
    pub contested_maps: u8,
    /// Quem vê joga a partida.
    pub yours: bool,
    pub task: Option<ViewerTask>,
    /// Onde o veto da série pronta está; `null` sem veto no torneio ou fora da
    /// série pronta.
    pub veto: Option<VetoStatus>,
    /// O mapa (a arena) do próximo jogo a enviar na série, quando o veto, a
    /// votação ou o sorteio já o deu; `null` antes disso, sem veto de mapas e
    /// com a série decidida.
    pub next_map: Option<MapRef>,
    /// Round robin: a série começou (veto ou mapa enviado) e ainda há mapa a
    /// jogar.
    pub in_progress: bool,
    /// Round robin: quem vê joga a série, que ainda não começou, e o adversário
    /// está jogando outra.
    pub opponent_playing: bool,
}

/// Um lado da partida.
#[derive(Serialize, ToSchema)]
pub struct MatchSide {
    /// O time deste lado (um dos `entrants` da chave), quando já se sabe.
    pub team_id: Option<i64>,
    /// Lado vazio: o outro avança sem jogar.
    pub bye: bool,
    /// De que partida vem quem ocupa o lado; `null` na primeira rodada.
    pub from: Option<SideSource>,
}

#[derive(Serialize, ToSchema)]
pub struct SideSource {
    pub outcome: SourceOutcome,
    pub match_key: String,
}

#[derive(Serialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceOutcome {
    Winner,
    Loser,
}

/// A série decidida.
#[derive(Serialize, ToSchema)]
pub struct MatchResult {
    pub walkover: bool,
    /// O lado que venceu; `null` na série par empatada, com metade dos mapas
    /// para cada lado.
    pub winner: Option<Side>,
    /// Mapas vencidos por cada lado; `null` no W.O.
    pub a_wins: Option<u8>,
    pub b_wins: Option<u8>,
    /// `maps` (os mapas confirmados deram a maioria ou empataram a série par),
    /// `organizer` (W.O.),
    /// `withdrawal` (W.O. automático); `opponent` e `deadline` só em séries
    /// enviadas inteiras, antes do envio mapa a mapa.
    pub confirmed_via: Confirmation,
    pub confirmed_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct Standing {
    /// O time (um dos `entrants` da chave).
    pub team_id: i64,
    pub rank: u16,
    pub tied: u16,
}

/// Uma partida com os lados, o detalhe por mapa e o histórico.
#[derive(Serialize, ToSchema)]
pub struct MatchDetail {
    pub summary: BracketMatch,
    /// A fase da partida, de 1 em diante.
    pub stage: u16,
    pub format: StageFormat,
    /// Rodadas da parte da chave desta partida (para nomear final,
    /// semifinal...). No round robin e no suíço, as rodadas da fase (no
    /// suíço, também as que ainda não foram pareadas).
    pub part_rounds: u16,
    pub status: TournamentStatus,
    /// Os times dos dois lados, quando já se sabe.
    pub a: Option<Team>,
    pub b: Option<Team>,
    /// Os mapas enviados, na ordem.
    pub games: Vec<GameDetail>,
    /// O próximo mapa que dá para enviar (série aberta, torneio em
    /// andamento); `null` quando não há.
    pub next_game: Option<i16>,
    /// O lado de quem vê, se joga a partida.
    pub viewer_side: Option<Side>,
    /// Quem vê joga a partida, e a regra do torneio deixa ele enviar, confirmar e
    /// contestar pelo time.
    pub can_report: bool,
    /// Do mais antigo ao mais novo.
    pub history: Vec<MatchEvent>,
    /// Quem vê é organizer ou owner (lança, corrige, resolve, dá W.O.).
    pub can_manage: bool,
    /// O veto de mapas e campeões; `null` sem veto no torneio, antes de a
    /// partida ficar pronta e na partida decidida sem veto.
    pub veto: Option<VetoView>,
    /// A hora da API ao montar a resposta: com ela, quem vê corrige o próprio
    /// relógio nos prazos (`veto.deadline`, `confirm_by`).
    pub now: DateTime<Utc>,
}

/// Um mapa da série.
#[derive(Serialize, ToSchema)]
pub struct GameDetail {
    pub number: i16,
    /// Quem jogou o mapa por cada lado (no duelo, o jogador; nos times, quem
    /// estava no time e na comunidade quando o mapa foi enviado).
    pub lineup_a: Vec<TeamMember>,
    pub lineup_b: Vec<TeamMember>,
    pub winner: Side,
    /// `null` só em série antiga, enviada sem detalhe por mapa.
    pub map: Option<MapRef>,
    /// Frags de cada lado, se vieram; de -999 a 999.
    pub a_frags: Option<i16>,
    pub b_frags: Option<i16>,
    pub status: GameStatus,
    /// Lado de quem enviou; `null` quando o organizador lançou.
    pub reported_side: Option<Side>,
    pub reported_at: DateTime<Utc>,
    /// Pendente: até quando o outro time confirma ou alguém contesta;
    /// depois, confirma sozinha.
    pub confirm_by: Option<DateTime<Utc>>,
    /// Contestada: o comentário de quem contestou.
    pub contest_comment: Option<String>,
    pub contested_at: Option<DateTime<Utc>>,
    /// Contestada: o apelido de quem contestou (pode ser do time de quem
    /// enviou); `null` se a conta foi excluída.
    pub contested_by: Option<String>,
    /// Quem vê pode contestar agora: a regra do torneio deixa e a partida está
    /// pendente e não foi ele quem enviou, ou confirmada e a contestação tardia
    /// ainda vale (até a próxima série de um dos lados começar).
    pub can_contest: bool,
    pub confirmed_via: Option<Confirmation>,
    pub confirmed_at: Option<DateTime<Utc>>,
    /// O que este mapa espera de quem vê (`respond`, `await_opponent`,
    /// `await_organizer`, `resolve`).
    pub task: Option<ViewerTask>,
}

#[derive(Serialize, ToSchema)]
pub struct MapRef {
    pub slug: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    /// Um jogador enviou o mapa (ou, antes do envio mapa a mapa, a série).
    Reported,
    /// O adversário (ou o organizador) confirmou.
    Confirmed,
    /// Confirmado pelo fim da janela de contestação.
    AutoConfirmed,
    /// O adversário contestou, com comentário.
    Contested,
    /// O organizador resolveu a contestação.
    Resolved,
    /// O organizador lançou o mapa direto.
    Entered,
    /// O organizador corrigiu um mapa já confirmado.
    Corrected,
    /// O organizador deu W.O. na série.
    Walkover,
    /// W.O. automático: um lado desistiu ou saiu da comunidade.
    AutoWalkover,
    /// Mapa anulado: veio depois da decisão da série, ou uma correção mudou
    /// quem joga a partida.
    Voided,
    /// O organizador anulou o mapa (pendente ou contestado) para ser jogado
    /// de novo.
    Replay,
    /// A série fechou: os mapas confirmados deram a maioria a um lado ou
    /// empataram a série par.
    Decided,
    /// Uma correção de mapa deixou a série sem decisão.
    Reopened,
}

#[derive(Serialize, ToSchema)]
pub struct MatchEvent {
    pub kind: EventKind,
    /// O mapa a que o evento se refere; `null`: a série inteira.
    pub game: Option<i16>,
    /// Quem agiu; `null` no que acontece sozinho (confirmação por tempo,
    /// W.O. automático, série decidida).
    pub actor: Option<EventActor>,
    pub at: DateTime<Utc>,
    /// O resultado que a ação deixou, quando ela define um.
    pub outcome: Option<EventOutcome>,
    pub comment: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct EventActor {
    pub user_id: i64,
    pub nickname: String,
    pub avatar_url: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct EventOutcome {
    /// O time vencedor (que pode não ser mais um dos lados, num envio
    /// anulado); `null` na série empatada.
    pub winner: Option<Team>,
    pub walkover: bool,
    /// Mapas do vencedor e do perdedor da série (na empatada, os de cada lado);
    /// `null` no W.O. e nos eventos de um mapa.
    pub winner_maps: Option<u8>,
    pub loser_maps: Option<u8>,
}

/// `{tournament}` e `{match}` de
/// `/tenants/{slug}/tournaments/{tournament}/matches/{match}`.
#[derive(Deserialize)]
pub struct MatchPath {
    pub tournament: String,
    #[serde(rename = "match")]
    pub key: MatchKey,
}

/// `{tournament}`, `{match}` e `{game}` de
/// `/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}`.
#[derive(Deserialize)]
pub struct GamePath {
    pub tournament: String,
    #[serde(rename = "match")]
    pub key: MatchKey,
    /// Número do mapa na série, a partir de 1.
    pub game: i16,
}

/// Um mapa da série: o mapa e o placar. O vencedor é quem fez mais frags.
#[derive(Deserialize, ToSchema)]
pub struct GameInput {
    /// Slug do mapa (`GET /maps`), sem repetir na série. Obrigatório, para o
    /// jogador e para o organizador (sem ele, 422 `match.map_required`).
    #[schema(example = "blood-covenant")]
    #[serde(default)]
    pub map: Option<String>,
    /// Frags do lado `a` e do lado `b`, de -999 a 999 (suicídio e queda tiram
    /// frag). Obrigatórios e sem empate: vence quem fez mais.
    #[schema(minimum = -999, maximum = 999)]
    pub a_frags: i16,
    #[schema(minimum = -999, maximum = 999)]
    pub b_frags: i16,
}

/// O mapa que o organizador lança ou corrige, já confirmado.
#[derive(Deserialize, ToSchema)]
pub struct SetGame {
    pub game: GameInput,
    /// Nota opcional no histórico, até 280 caracteres.
    #[serde(default)]
    pub comment: Option<String>,
}

/// Anular o último mapa para ser jogado de novo.
#[derive(Deserialize, ToSchema)]
pub struct ReplayGame {
    /// Nota opcional no histórico, até 280 caracteres.
    #[serde(default)]
    pub comment: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct ContestReport {
    /// O que está errado, de 1 a 280 caracteres.
    pub comment: String,
}

/// O resultado da série que o organizador define: W.O.
#[derive(Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ResultInput {
    Walkover {
        /// Lado que avança.
        winner: Side,
    },
}

#[derive(Deserialize, ToSchema)]
pub struct SetResult {
    pub result: ResultInput,
    /// Nota opcional no histórico, até 280 caracteres.
    #[serde(default)]
    pub comment: Option<String>,
}
