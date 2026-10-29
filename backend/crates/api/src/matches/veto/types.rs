//! DTOs do veto: o que a página da partida mostra e o que as ações recebem.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    matches::{result::Side, types::MapRef},
    tournaments::MapMode,
};

/// Onde o veto da partida está (só na partida pronta de um torneio com veto).
#[derive(Serialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VetoStatus {
    /// Esperando os dois clicarem em "Pronto" (ou o organizador forçar).
    WaitingReady,
    /// Algum passo do jogo em andamento espera um lado.
    InProgress,
    /// O veto do jogo em andamento acabou: espera o envio dele.
    Done,
}

#[derive(Serialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VetoAction {
    Ban,
    Pick,
    Vote,
}

#[derive(Serialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VetoTarget {
    Map,
    Champion,
}

/// O passo esperado agora.
#[derive(Serialize, ToSchema)]
pub struct VetoTurn {
    pub action: VetoAction,
    pub target: VetoTarget,
    /// Quem age: um lado, ou os dois na votação enquanto nenhum votou.
    pub sides: Vec<Side>,
}

#[derive(Serialize, ToSchema, Clone)]
pub struct ChampionRef {
    pub slug: String,
    pub name: String,
}

/// Um passo do veto de mapas (modo veto).
#[derive(Serialize, ToSchema)]
pub struct VetoMapStep {
    pub side: Side,
    pub action: VetoAction,
    pub map: MapRef,
}

#[derive(Serialize, ToSchema)]
pub struct VetoBan {
    pub side: Side,
    pub champion: ChampionRef,
}

/// O veto de um jogo da série.
#[derive(Serialize, ToSchema)]
pub struct VetoGame {
    pub game: i16,
    /// O mapa do jogo, quando já saiu (no modo desligado, nunca).
    pub map: Option<MapRef>,
    /// Quem escolheu o mapa no veto; `null` no decisivo e fora do modo veto.
    pub picked_by: Option<Side>,
    /// Votação: os mapas oferecidos e o voto de cada lado.
    pub offered: Vec<MapRef>,
    pub vote_a: Option<String>,
    pub vote_b: Option<String>,
    /// Quem começa os campeões.
    pub starter: Option<Side>,
    pub bans: Vec<VetoBan>,
    pub pick_a: Option<ChampionRef>,
    pub pick_b: Option<ChampionRef>,
}

/// Um campeão do torneio no jogo em andamento, com o que o tira das opções.
#[derive(Serialize, ToSchema)]
pub struct VetoChampion {
    pub slug: String,
    pub name: String,
    /// Banido neste mapa (vale para os dois), e por quem.
    pub banned_by: Option<Side>,
    /// A regra de não repetição o tira de cada lado (já usado na série ou,
    /// pelo escopo da regra, em outra partida).
    pub used_by_a: bool,
    pub used_by_b: bool,
}

#[derive(Serialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VetoEventKind {
    /// Um jogador disse que está pronto.
    Ready,
    /// O veto começou (`actor`: o organizador que forçou).
    Begin,
    /// O jogo começou (o anterior foi enviado).
    Game,
    Ban,
    Pick,
    Vote,
    /// O último passo foi desfeito (pelo organizador, ou aceito pelo
    /// adversário).
    Undo,
    /// Um jogador pediu para desfazer o último passo.
    UndoRequest,
    /// O adversário recusou o pedido.
    UndoDeclined,
}

/// Um evento do histórico do veto.
#[derive(Serialize, ToSchema)]
pub struct VetoEvent {
    pub kind: VetoEventKind,
    pub game: Option<i16>,
    /// Quem agiu (ou, no começo e no início de um jogo, quem começa).
    pub side: Option<Side>,
    pub map: Option<MapRef>,
    pub champion: Option<ChampionRef>,
    /// Escolha sorteada pelo fim do tempo.
    pub timeout: bool,
    /// Apelido de quem agiu; `null` no que acontece sozinho.
    pub actor: Option<String>,
    pub at: DateTime<Utc>,
}

/// O veto da partida.
#[derive(Serialize, ToSchema)]
pub struct VetoView {
    pub map_mode: MapMode,
    pub champion_phase: bool,
    /// Segundos de cada passo.
    pub step_seconds: i32,
    /// O pool de mapas do torneio, na ordem.
    pub pool: Vec<MapRef>,
    pub status: VetoStatus,
    pub ready_a: bool,
    pub ready_b: bool,
    /// O jogo em andamento.
    pub game: Option<i16>,
    pub turn: Option<VetoTurn>,
    /// Até quando o passo esperado pode ser feito; depois, é sorteado.
    pub deadline: Option<DateTime<Utc>>,
    /// Modo veto: os passos do veto de mapas, na ordem.
    pub map_steps: Vec<VetoMapStep>,
    /// Modo veto: os mapas que ainda não saíram.
    pub remaining: Vec<MapRef>,
    pub games: Vec<VetoGame>,
    /// O que quem vê pode escolher agora (slugs de mapas ou de campeões);
    /// vazio se não é a vez dele.
    pub options: Vec<String>,
    /// Os campeões do torneio no jogo em andamento (com a fase de campeões).
    pub champions: Vec<VetoChampion>,
    /// O lado que pediu para desfazer o último passo, esperando o outro.
    pub undo_request: Option<Side>,
    /// Há passo para desfazer no jogo em andamento.
    pub can_undo: bool,
    /// Os jogadores podem enviar o próximo mapa (o veto dele acabou).
    pub reportable: bool,
    /// Do mais antigo ao mais novo.
    pub history: Vec<VetoEvent>,
}

/// Um passo: um mapa ou um campeão, pelo slug.
#[derive(Deserialize, ToSchema)]
pub struct VetoChoice {
    #[serde(default)]
    pub map: Option<String>,
    #[serde(default)]
    pub champion: Option<String>,
}

/// A resposta ao pedido de desfazer do adversário.
#[derive(Deserialize, ToSchema)]
pub struct UndoAnswer {
    pub accept: bool,
}
