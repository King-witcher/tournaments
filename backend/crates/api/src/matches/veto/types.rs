//! DTOs do veto: o que a página da série mostra e o que as ações recebem.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    matches::{result::Side, types::MapRef},
    tournaments::{MapMode, PickOrder},
};

/// Onde o veto da série está (só na série pronta de um torneio com veto).
#[derive(Serialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VetoStatus {
    /// Esperando os dois lados clicarem em "Pronto" (ou o organizador
    /// forçar).
    WaitingReady,
    /// Algum passo da partida em andamento espera um lado.
    InProgress,
    /// O veto da partida em andamento acabou: espera o envio dela.
    Done,
}

#[derive(Serialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VetoAction {
    Ban,
    Pick,
    Vote,
    /// Um jogador escolhe o próprio campeão. Sem `pick_for`, é a janela: todos
    /// ao mesmo tempo, cada um para cada partida (`team_pick_seconds`). Com
    /// `pick_for`, é a ordem alternada: qualquer um do lado da vez que ainda não
    /// escolheu para aquela partida, com o tempo de um passo.
    TeamPick,
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
    /// Quem age: um lado (qualquer jogador do time), ou os dois na votação
    /// enquanto nenhum votou e na janela simultânea enquanto falta alguém de
    /// cada um.
    pub sides: Vec<Side>,
    /// Pick de campeão com vez: a partida a que ele se destina.
    pub pick_for: Option<i16>,
}

/// Um jogador que ainda falta escolher, no pick por jogador.
#[derive(Serialize, ToSchema)]
pub struct VetoPlayer {
    pub side: Side,
    pub user_id: i64,
    /// Apelido; vazio quando a conta foi excluída.
    pub nickname: String,
    /// As partidas para as quais ele ainda não escolheu.
    pub games: Vec<i16>,
}

/// O campeão de um jogador numa partida, no pick por jogador.
#[derive(Serialize, ToSchema)]
pub struct VetoTeamPick {
    pub side: Side,
    pub user_id: i64,
    pub nickname: String,
    pub champion: ChampionRef,
    /// Sorteado pelo fim do tempo.
    pub timeout: bool,
}

/// Às cegas: um jogador de outro lado já escolheu para a partida, sem dizer o
/// quê, até a janela fechar.
#[derive(Serialize, ToSchema)]
pub struct VetoHiddenPick {
    pub side: Side,
    pub user_id: i64,
    pub nickname: String,
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

/// Um ban de campeão da série (vale para os dois lados em todas as
/// partidas).
#[derive(Serialize, ToSchema)]
pub struct VetoBan {
    pub side: Side,
    pub champion: ChampionRef,
    /// Sorteado pelo fim do tempo.
    pub timeout: bool,
}

/// Uma partida da série: o mapa e os campeões escolhidos para ela.
#[derive(Serialize, ToSchema)]
pub struct VetoGame {
    pub game: i16,
    /// O mapa da partida, quando já saiu (no modo desligado, nunca).
    pub map: Option<MapRef>,
    /// Quem escolheu o mapa no veto; `null` no decisivo e fora do modo veto.
    pub picked_by: Option<Side>,
    /// Votação: os mapas oferecidos e o voto de cada lado.
    pub offered: Vec<MapRef>,
    pub vote_a: Option<String>,
    pub vote_b: Option<String>,
    /// Duelo alternado: o campeão de cada lado.
    pub pick_a: Option<ChampionRef>,
    pub pick_b: Option<ChampionRef>,
    /// No pick por jogador: o campeão de cada um, na ordem em que escolheram (só
    /// o que conta: numa partida enviada, de quem a jogou). Às cegas, com a
    /// janela aberta, só os do lado de quem vê.
    pub team_picks: Vec<VetoTeamPick>,
    /// Às cegas, com a janela aberta: quem já escolheu, sem o campeão, dos lados
    /// que não são o de quem vê.
    pub hidden_picks: Vec<VetoHiddenPick>,
}

/// Um campeão ativo do torneio, com o ban da série.
#[derive(Serialize, ToSchema)]
pub struct VetoChampion {
    pub slug: String,
    pub name: String,
    /// Banido na série (vale para os dois), e por quem.
    pub banned_by: Option<Side>,
}

/// Por que um campeão não está nas opções de quem escolhe.
#[derive(Serialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VetoUnavailableReason {
    Banned,
    /// Um colega já o escolheu para a partida (sem repetir no time).
    Teammate,
    /// Um adversário já o escolheu para a partida (sem espelho).
    Opponent,
    /// A regra de não repetição: escolhido para outra partida da série ou
    /// usado fora dela.
    Repeat,
}

#[derive(Serialize, ToSchema)]
pub struct VetoUnavailable {
    pub slug: String,
    pub reason: VetoUnavailableReason,
}

/// O que cedeu para quem vê ter campeão (cada um cede sozinho, na hora em
/// que escolhe).
#[derive(Serialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VetoGaveWay {
    /// A regra de não repetição: com ela, não sobrava nenhum; as opções
    /// trazem os que ela tira.
    Rule,
    /// Também a da partida (espelho, repetir no time): nem sem a regra
    /// sobrava um.
    Game,
}

/// Um pick que quem vê pode fazer agora: no duelo alternado, o da vez dele; no
/// pick por jogador, cada partida em que ele falta e pode escolher agora.
#[derive(Serialize, ToSchema)]
pub struct VetoPickSlot {
    pub game: i16,
    /// Os slugs que ele pode escolher.
    pub options: Vec<String>,
    /// Os campeões que não estão nas opções, com o motivo (sem contar o que
    /// cedeu).
    pub unavailable: Vec<VetoUnavailable>,
    /// As opções vieram com algo cedido; `null` se nada cedeu.
    pub gave_way: Option<VetoGaveWay>,
}

#[derive(Serialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VetoEventKind {
    /// Um lado está pronto (`actor`: o jogador que clicou).
    Ready,
    /// O veto começou (`actor`: o organizador que forçou).
    Begin,
    /// A partida seguinte começou (a anterior foi enviada).
    Game,
    Ban,
    Pick,
    Vote,
    /// O último passo foi desfeito (pelo organizador, ou aceito pelo
    /// adversário).
    Undo,
    /// Um lado pediu para desfazer o último passo.
    UndoRequest,
    /// O adversário recusou o pedido.
    UndoDeclined,
}

/// Um evento do histórico do veto.
#[derive(Serialize, ToSchema)]
pub struct VetoEvent {
    pub kind: VetoEventKind,
    /// A partida em andamento quando aconteceu.
    pub game: Option<i16>,
    /// Pick de campeão: a partida a que ele se destina.
    pub pick_game: Option<i16>,
    /// Quem agiu (ou, no começo, quem começa o veto de mapas).
    pub side: Option<Side>,
    pub map: Option<MapRef>,
    pub champion: Option<ChampionRef>,
    /// Escolha sorteada pelo fim do tempo.
    pub timeout: bool,
    /// Apelido de quem agiu; `null` no que acontece sozinho.
    pub actor: Option<String>,
    /// No pick por jogador, o apelido do jogador (também no sorteado).
    pub player: Option<String>,
    pub at: DateTime<Utc>,
}

/// O veto da série.
#[derive(Serialize, ToSchema)]
pub struct VetoView {
    pub map_mode: MapMode,
    /// Draft de campeões no site.
    pub champion_phase: bool,
    /// A ordem das escolhas de campeão.
    pub pick_order: PickOrder,
    /// Jogadores por lado.
    pub team_size: i16,
    /// Segundos de cada passo.
    pub step_seconds: i32,
    /// Segundos da janela simultânea de campeões; `null` sem janela.
    pub team_pick_seconds: Option<i32>,
    /// O pool de mapas do torneio, na ordem.
    pub pool: Vec<MapRef>,
    pub status: VetoStatus,
    pub ready_a: bool,
    pub ready_b: bool,
    /// A partida em andamento.
    pub game: Option<i16>,
    pub turn: Option<VetoTurn>,
    /// A vez que esta resposta mostra (o id do evento que a abriu; `null`
    /// antes do começo). Muda a cada passo feito, sorteado ou desfeito e a
    /// cada partida nova, mesmo quando o mesmo lado age de novo; o voto que
    /// espera o do outro e as escolhas da janela simultânea não a mudam.
    /// Mande-a em `step` nas ações (passo, desfazer, pedir e responder o
    /// desfazer): se a vez mudou antes de a ação chegar, ela é recusada (409
    /// `veto.step_changed`) em vez de cair no passo seguinte.
    pub step: Option<i64>,
    /// Até quando o passo esperado pode ser feito (o prazo que o cronômetro
    /// mostra). A API ainda aceita a ação até 2 s depois; então ela é
    /// sorteada.
    pub deadline: Option<DateTime<Utc>>,
    /// Modo veto: os passos do veto de mapas, na ordem.
    pub map_steps: Vec<VetoMapStep>,
    /// Modo veto: os mapas que ainda não saíram.
    pub remaining: Vec<MapRef>,
    /// Os bans de campeão da série, na ordem.
    pub bans: Vec<VetoBan>,
    /// Cada partida da série, da 1 à última (com o draft, os campeões de
    /// todas aparecem antes da partida 1).
    pub games: Vec<VetoGame>,
    /// O que quem vê pode escolher no passo do lado dele agora (slugs de
    /// mapas ou de campeões: veto de mapas, voto, ban, pick do duelo alternado);
    /// vazio se não é a vez do lado dele e no pick por jogador (`picks`).
    pub options: Vec<String>,
    /// Os picks de campeão que quem vê pode fazer agora, com o motivo de cada
    /// campeão fora.
    pub picks: Vec<VetoPickSlot>,
    /// No pick por jogador: quem ainda falta, e em quais partidas.
    pub missing: Vec<VetoPlayer>,
    /// Os campeões ativos do torneio (com o draft no site), com os bans.
    pub champions: Vec<VetoChampion>,
    /// O lado que pediu para desfazer o último passo, esperando o outro.
    pub undo_request: Option<Side>,
    /// Há passo para desfazer na partida em andamento.
    pub can_undo: bool,
    /// Os jogadores podem enviar a próxima partida (o veto dela acabou).
    pub reportable: bool,
    /// Do mais antigo ao mais novo. Às cegas, com a janela aberta, sem as
    /// escolhas dos lados que não são o de quem vê.
    pub history: Vec<VetoEvent>,
}

/// Um passo: um mapa ou um campeão, pelo slug.
#[derive(Deserialize, ToSchema)]
pub struct VetoChoice {
    #[serde(default)]
    pub map: Option<String>,
    #[serde(default)]
    pub champion: Option<String>,
    /// Na janela simultânea: a partida para a qual o campeão é escolhido. Nos
    /// picks com vez, vale a que a vez diz, e outra é 409 `veto.step_changed`.
    #[serde(default)]
    pub game: Option<i16>,
    /// A vez que a tela mostrava (`VetoView::step`); sem ela, vale a de
    /// agora.
    #[serde(default)]
    pub step: Option<i64>,
}

/// A vez que a tela de quem age mostrava (`VetoView::step`), no desfazer e
/// no pedido de desfazer. O corpo é opcional; sem ele, vale a de agora.
#[derive(Deserialize, ToSchema, Default)]
pub struct VetoSeen {
    #[serde(default)]
    pub step: Option<i64>,
}

/// A resposta ao pedido de desfazer do adversário.
#[derive(Deserialize, ToSchema)]
pub struct UndoAnswer {
    pub accept: bool,
    /// A vez que a tela mostrava (`VetoView::step`); sem ela, vale a de
    /// agora.
    #[serde(default)]
    pub step: Option<i64>,
}
