//! DTOs da sala de veto. O veto em si é o `VetoView` da série de um torneio.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    matches::{Side, veto::VetoView},
    tournaments::{NoRepeatWho, VetoInput, VetoOrder},
};

#[derive(Deserialize)]
pub struct VetoRoomPath {
    pub token: String,
}

/// Uma sala nova.
#[derive(Deserialize, ToSchema)]
pub struct NewVetoRoom {
    /// Jogadores por lado: 1 (duelo), 2, 3 ou 4.
    #[schema(example = 1)]
    pub team_size: i16,
    /// Melhor de 1, 3 ou 5.
    #[schema(example = 3)]
    pub best_of: i16,
    /// Mapas e tempos. `map_mode`: `veto` (padrão) ou `off`; votação e sorteio
    /// dependem do envio de cada partida, que a sala não tem. O resto como no
    /// torneio: ausente, o padrão.
    #[serde(default)]
    pub veto: Option<VetoInput>,
    /// Campeões, quando `veto.champion_phase` está ligado; ausente, o padrão.
    #[serde(default)]
    pub champion_rules: Option<VetoRoomChampionRules>,
}

/// As regras de campeões da sala; a repetição vale dentro da série.
#[derive(Deserialize, ToSchema, Debug, Clone, Copy)]
pub struct VetoRoomChampionRules {
    /// Bans de campeão de cada lado, de 0 a 3 (padrão 1).
    #[serde(default = "default_bans")]
    #[schema(required = false, default = 1, example = 1)]
    pub bans_per_series: u8,
    /// Adversários podem usar o mesmo campeão na mesma partida (padrão sim).
    #[serde(default = "default_mirror")]
    #[schema(required = false, default = true)]
    pub mirror: bool,
    /// Nos times: colegas podem usar o mesmo campeão na mesma partida (padrão
    /// não).
    #[serde(default)]
    pub same_team: bool,
    /// Quem não repete campeão entre as partidas da série; `null`: pode
    /// repetir.
    #[serde(default)]
    pub no_repeat: Option<NoRepeatWho>,
}

const fn default_bans() -> u8 {
    1
}

const fn default_mirror() -> bool {
    true
}

impl Default for VetoRoomChampionRules {
    fn default() -> Self {
        Self {
            bans_per_series: default_bans(),
            mirror: default_mirror(),
            same_team: false,
            no_repeat: None,
        }
    }
}

/// A sala criada: `token` é o link de gestão.
#[derive(Serialize, ToSchema)]
pub struct VetoRoomCreated {
    pub token: String,
}

/// Quem entrou por este link.
#[derive(Serialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VetoRoomRole {
    /// O link de gestão: vê todos os links e o andamento.
    Manager,
    Spectator,
    Player,
}

/// Um lugar da sala.
#[derive(Serialize, ToSchema, Debug, Clone)]
pub struct VetoRoomSeat {
    /// O jogador no veto (`VetoTeamPick.user_id`, `VetoPlayer.user_id`).
    pub id: i64,
    pub side: Side,
    /// A posição no lado, de 1 em diante.
    pub position: i16,
    /// `null`: ninguém entrou neste lugar.
    pub name: Option<String>,
}

/// O link de um lugar.
#[derive(Serialize, ToSchema)]
pub struct VetoRoomSeatLink {
    pub seat: i64,
    pub token: String,
}

/// Todos os links da sala: só o de gestão os vê.
#[derive(Serialize, ToSchema)]
pub struct VetoRoomLinks {
    pub manage: String,
    pub spectator: String,
    pub seats: Vec<VetoRoomSeatLink>,
}

/// A sala como este link a vê.
#[derive(Serialize, ToSchema)]
pub struct VetoRoomDetail {
    /// A hora da API, para o cronômetro.
    pub now: DateTime<Utc>,
    /// Quando a sala deixa de existir: 2 h depois de criada, se ninguém
    /// começou; 24 h depois do início.
    pub expires_at: DateTime<Utc>,
    pub role: VetoRoomRole,
    /// O lugar deste link (`player`). Com `name` nulo, falta entrar.
    pub seat: Option<VetoRoomSeat>,
    pub team_size: i16,
    pub best_of: i16,
    pub bans_per_series: i16,
    pub order: VetoOrder,
    pub no_repeat: Option<NoRepeatWho>,
    pub seats: Vec<VetoRoomSeat>,
    /// Só para o link de gestão.
    pub links: Option<VetoRoomLinks>,
    /// O veto, do ponto de vista deste link. `user_id` e os ids do histórico são
    /// lugares; nomes vazios são lugares em que ninguém entrou.
    pub veto: VetoView,
}

/// Quem entra num lugar.
#[derive(Deserialize, ToSchema, Default)]
pub struct JoinVetoRoom {
    /// O nome, de 1 a 32 caracteres, de quem não está logado; logado, vale o da
    /// conta.
    #[serde(default)]
    pub name: Option<String>,
}
