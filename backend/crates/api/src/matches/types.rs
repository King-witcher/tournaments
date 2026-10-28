use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::result::{Confirmation, ResultStatus, Side};
use crate::tournaments::{StageFormat, TournamentStatus};

/// A chave da fase em jogo, refeita dos resultados confirmados.
#[derive(Serialize, ToSchema)]
pub struct TournamentBracket {
    pub format: StageFormat,
    /// Status do torneio depois das confirmações por tempo.
    pub status: TournamentStatus,
    /// Participantes na ordem dos seeds.
    pub entrants: Vec<BracketEntrant>,
    /// Todas as partidas, por parte da chave, rodada e índice.
    pub matches: Vec<BracketMatch>,
    /// Colocações já definidas (os eliminados e, no fim, campeão e vice).
    /// Empatados dividem a faixa: `rank` 5 com `tied` 4 é 5º–8º.
    pub standings: Vec<Standing>,
    /// Janela de contestação em minutos.
    pub contest_minutes: i32,
    /// Resultados (pendentes, contestados ou confirmados) que desfazer o
    /// início apaga.
    pub results: i64,
}

#[derive(Serialize, ToSchema)]
pub struct BracketEntrant {
    pub user_id: i64,
    /// Apelido no tenant.
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub seed: i32,
    /// Saiu da comunidade (ou foi removido); continua na chave como histórico.
    pub left: bool,
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

/// Estado da partida na chave (só resultados confirmados contam).
#[derive(Serialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MatchState {
    /// Algum lado ainda depende de outra partida.
    Pending,
    /// Os dois lados são conhecidos: dá para jogar (o resultado pode estar
    /// enviado ou contestado).
    Ready,
    /// Um lado vazio: o outro avança sem jogar.
    Bye,
    /// Resultado confirmado.
    Completed,
    /// Reset da grande final que não precisou acontecer.
    NotNeeded,
}

/// O que a partida espera de quem vê.
#[derive(Serialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViewerTask {
    /// Enviar o placar (joga a partida e ninguém enviou).
    Report,
    /// Confirmar ou contestar o placar que o adversário enviou.
    Respond,
    /// Enviou e espera o adversário (ou o fim da janela).
    AwaitOpponent,
    /// Contestada: espera o organizador.
    AwaitOrganizer,
    /// Organizer ou owner: resolver a contestação.
    Resolve,
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
    /// Resultado vigente, se houver.
    pub result: Option<MatchResult>,
    /// Quem vê joga a partida.
    pub yours: bool,
    pub task: Option<ViewerTask>,
}

/// Um lado da partida.
#[derive(Serialize, ToSchema)]
pub struct MatchSide {
    /// Quem joga deste lado, quando já se sabe.
    pub user_id: Option<i64>,
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

/// O resultado vigente de uma partida.
#[derive(Serialize, ToSchema)]
pub struct MatchResult {
    pub status: ResultStatus,
    pub walkover: bool,
    pub winner: Side,
    /// Mapas vencidos por cada lado; `null` no W.O.
    pub a_wins: Option<u8>,
    pub b_wins: Option<u8>,
    /// Lado do jogador que enviou; `null` quando o organizador lançou.
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
}

#[derive(Serialize, ToSchema)]
pub struct Standing {
    pub user_id: i64,
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
    pub a: Option<BracketEntrant>,
    pub b: Option<BracketEntrant>,
    /// Detalhe por mapa do resultado vigente, na ordem jogada (pode faltar).
    pub games: Vec<GameDetail>,
    /// Do mais antigo ao mais novo.
    pub history: Vec<MatchEvent>,
    /// Quem vê é organizer ou owner (lança, corrige, resolve, dá W.O.).
    pub can_manage: bool,
}

#[derive(Serialize, ToSchema)]
pub struct GameDetail {
    pub number: i16,
    pub map: MapRef,
    pub a_frags: i16,
    pub b_frags: i16,
}

#[derive(Serialize, ToSchema)]
pub struct MapRef {
    pub slug: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    /// Um jogador enviou o placar.
    Reported,
    /// O adversário (ou o organizador) confirmou.
    Confirmed,
    /// Confirmado pelo fim da janela de contestação.
    AutoConfirmed,
    /// O adversário contestou, com comentário.
    Contested,
    /// O organizador resolveu a contestação com um placar.
    Resolved,
    /// O organizador lançou o placar direto.
    Entered,
    /// O organizador corrigiu um resultado já confirmado.
    Corrected,
    /// O organizador deu W.O.
    Walkover,
    /// Resultado pendente anulado porque uma correção mudou os lados.
    Voided,
}

#[derive(Serialize, ToSchema)]
pub struct MatchEvent {
    pub kind: EventKind,
    /// Quem agiu; `null` na confirmação por tempo.
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
    /// `user_id` do vencedor.
    pub winner: i64,
    /// Apelido do vencedor (que pode não ser mais um dos lados, num envio
    /// anulado).
    pub winner_nickname: String,
    pub walkover: bool,
    /// Mapas do vencedor e do perdedor; `null` no W.O.
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

/// Um mapa do detalhe enviado.
#[derive(Deserialize, ToSchema)]
pub struct GameInput {
    /// Slug do mapa (`GET /maps`).
    #[schema(example = "blood-covenant")]
    pub map: String,
    /// Frags do lado `a` e do lado `b`, de 0 a 999; não pode empatar.
    pub a_frags: u16,
    pub b_frags: u16,
}

/// Placar da série que um jogador envia.
#[derive(Deserialize, ToSchema)]
pub struct ReportScore {
    /// Mapas vencidos pelo lado `a` e pelo lado `b` (ex.: 2 e 1 numa md3).
    pub a_wins: u8,
    pub b_wins: u8,
    /// Detalhe opcional por mapa, na ordem jogada: um item por mapa, com os
    /// vencedores batendo com o placar.
    #[serde(default)]
    pub games: Vec<GameInput>,
}

#[derive(Deserialize, ToSchema)]
pub struct ContestReport {
    /// O que está errado, de 1 a 280 caracteres.
    pub comment: String,
}

/// O resultado que o organizador define: série jogada ou W.O.
#[derive(Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ResultInput {
    Played {
        a_wins: u8,
        b_wins: u8,
        #[serde(default)]
        games: Vec<GameInput>,
    },
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
