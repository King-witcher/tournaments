use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{
    games::GameStatus,
    result::{Confirmation, Side},
    veto::{VetoStatus, VetoView},
};
use crate::{
    teams::Team,
    tournaments::{StageFormat, TournamentStatus},
};

/// A chave da fase em jogo, refeita dos resultados confirmados.
#[derive(Serialize, ToSchema)]
pub struct TournamentBracket {
    pub format: StageFormat,
    /// Status do torneio depois das confirmações por tempo.
    pub status: TournamentStatus,
    /// Os times da chave, na ordem dos seeds.
    pub entrants: Vec<Team>,
    /// Todas as partidas, por parte da chave, rodada e índice.
    pub matches: Vec<BracketMatch>,
    /// Colocações já definidas (os eliminados e, no fim, campeão e vice).
    /// Empatados dividem a faixa: `rank` 5 com `tied` 4 é 5º–8º.
    pub standings: Vec<Standing>,
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
    /// Mapa contestado: espera o organizador.
    AwaitOrganizer,
    /// Organizer ou owner: resolver um mapa contestado.
    Resolve,
    /// Clicar em "Pronto" para o veto começar.
    Ready,
    /// Fazer o passo do veto que está na vez dele.
    Veto,
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
    /// Onde o veto da partida pronta está; `null` sem veto no torneio ou fora
    /// da partida pronta.
    pub veto: Option<VetoStatus>,
}

#[derive(Serialize, ToSchema)]
pub struct BracketMatch {
    /// Identificador na rota: `wb-1-3`, `lb-2-1`, `gf-1-1`.
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
    /// Mapas enviados esperando o adversário (ou o fim da janela).
    pub pending_maps: u8,
    /// Mapas contestados, esperando o organizador.
    pub contested_maps: u8,
    /// Quem vê joga a partida.
    pub yours: bool,
    pub task: Option<ViewerTask>,
    /// Onde o veto da partida pronta está; `null` sem veto no torneio ou fora
    /// da partida pronta.
    pub veto: Option<VetoStatus>,
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
    pub winner: Side,
    /// Mapas vencidos por cada lado; `null` no W.O.
    pub a_wins: Option<u8>,
    pub b_wins: Option<u8>,
    /// `maps` (os mapas confirmados deram a maioria), `organizer` (W.O.),
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
    pub format: StageFormat,
    /// Rodadas da parte da chave desta partida (para nomear final,
    /// semifinal...).
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
    /// Do mais antigo ao mais novo.
    pub history: Vec<MatchEvent>,
    /// Quem vê é organizer ou owner (lança, corrige, resolve, dá W.O.).
    pub can_manage: bool,
    /// O veto de mapas e campeões; `null` sem veto no torneio, antes de a
    /// partida ficar pronta e na partida decidida sem veto.
    pub veto: Option<VetoView>,
}

/// Um mapa da série.
#[derive(Serialize, ToSchema)]
pub struct GameDetail {
    pub number: i16,
    pub winner: Side,
    /// `null` só em série antiga, enviada sem detalhe por mapa.
    pub map: Option<MapRef>,
    /// Frags de cada lado, se vieram.
    pub a_frags: Option<i16>,
    pub b_frags: Option<i16>,
    pub status: GameStatus,
    /// Lado de quem enviou; `null` quando o organizador lançou.
    pub reported_side: Option<Side>,
    pub reported_at: DateTime<Utc>,
    /// Pendente: até quando o adversário confirma ou contesta; depois,
    /// confirma sozinho.
    pub confirm_by: Option<DateTime<Utc>>,
    /// Contestado: o comentário de quem contestou.
    pub contest_comment: Option<String>,
    pub contested_at: Option<DateTime<Utc>>,
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
    /// A série fechou: os mapas confirmados deram a maioria a um lado.
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
    /// anulado).
    pub winner: Team,
    pub walkover: bool,
    /// Mapas do vencedor e do perdedor da série; `null` no W.O. e nos
    /// eventos de um mapa.
    pub winner_maps: Option<u8>,
    pub loser_maps: Option<u8>,
}

/// `{tournament}` e `{match}` de
/// `/tenants/{slug}/tournaments/{tournament}/matches/{match}`.
#[derive(Deserialize)]
pub struct MatchPath {
    pub tournament: String,
    #[serde(rename = "match")]
    pub key: String,
}

/// `{tournament}`, `{match}` e `{game}` de
/// `/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}`.
#[derive(Deserialize)]
pub struct GamePath {
    pub tournament: String,
    #[serde(rename = "match")]
    pub key: String,
    /// Número do mapa na série, a partir de 1.
    pub game: i16,
}

/// Um mapa da série: quem venceu, o mapa e os frags.
#[derive(Deserialize, ToSchema)]
pub struct GameInput {
    pub winner: Side,
    /// Slug do mapa (`GET /maps`), sem repetir na série. Obrigatório no envio
    /// do jogador; o organizador pode deixar sem, numa série antiga.
    #[schema(example = "blood-covenant")]
    #[serde(default)]
    pub map: Option<String>,
    /// Frags do lado `a` e do lado `b`, de 0 a 999, opcionais (os dois ou
    /// nenhum); sem empate e com mais frags para quem venceu.
    #[serde(default)]
    pub a_frags: Option<u16>,
    #[serde(default)]
    pub b_frags: Option<u16>,
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
