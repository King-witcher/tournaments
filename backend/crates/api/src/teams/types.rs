use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Um time do torneio: quem ocupa um lado da chave. No duelo, o time de um
/// jogador, sem nome (a tela mostra o apelido dele).
#[derive(Serialize, ToSchema, Debug, Clone)]
pub struct Team {
    pub id: i64,
    /// Nome do time; `null` no duelo e nos times sem nome (a tela mostra o
    /// apelido do único jogador, ou "Time de {capitão}").
    pub name: Option<String>,
    /// Posição no seeding (1 = melhor); `null` sem seed. Depois de iniciar,
    /// 1..N entre os times da chave.
    pub seed: Option<i32>,
    /// `user_id` do capitão, um dos `members` (no duelo, o único).
    pub captain: i64,
    /// Desistiu do torneio em andamento: as partidas prontas dele viram W.O.
    /// para o adversário.
    pub withdrawn: bool,
    /// Os jogadores: o capitão primeiro, depois na ordem de inscrição.
    pub members: Vec<TeamMember>,
}

/// Um jogador de um time.
#[derive(Serialize, ToSchema, Debug, Clone)]
pub struct TeamMember {
    pub user_id: i64,
    /// Apelido no tenant; vazio quando a conta foi excluída (a tela mostra
    /// "Conta excluída").
    pub nickname: String,
    pub avatar_url: Option<String>,
    /// Saiu da comunidade (ou foi removido); continua no time como histórico.
    /// O time sem nenhum jogador na comunidade está fora do torneio.
    pub left: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct SeedingOrder {
    /// Todos os times elegíveis, cada um uma vez, do seed 1 em diante.
    pub team_ids: Vec<i64>,
}

/// `{tournament}` e `{team}` de
/// `/tenants/{slug}/tournaments/{tournament}/teams/{team}` (o slug do tenant
/// fica com `TenantMember`).
#[derive(Deserialize)]
pub struct TeamPath {
    pub tournament: String,
    /// Id do time.
    pub team: i64,
}

/// A formação dos times de um torneio: os times e quem está sem time (os
/// reservas). No duelo, cada inscrito é um time de um e não há reservas.
#[derive(Serialize, ToSchema)]
pub struct TeamFormation {
    /// Jogadores por time (1 no duelo).
    pub team_size: i32,
    /// O torneio tem check-in: só quem fez entra num time que vai para a
    /// chave.
    pub has_check_in: bool,
    /// Os times, pelo seed e, sem seed, na ordem de criação.
    pub teams: Vec<Team>,
    /// Os inscritos sem time, na ordem de inscrição (no TDM em andamento,
    /// também quem foi substituído).
    pub reserves: Vec<FormationPlayer>,
    /// `user_id` de quem fez check-in, entre todos os inscritos.
    pub checked_in: Vec<i64>,
}

/// Um inscrito sem time.
#[derive(Serialize, ToSchema)]
pub struct FormationPlayer {
    pub user_id: i64,
    /// Apelido no tenant; vazio quando a conta foi excluída.
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub registered_at: DateTime<Utc>,
    /// Saiu da comunidade (ou foi removido).
    pub left: bool,
}

/// Como sortear os times.
#[derive(Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DrawMethod {
    /// Aleatório: os jogadores embaralhados pela seed; o capitão, sorteado.
    Random,
    /// Balanceado: zigue-zague pelo rating da comunidade (o inicial para
    /// quem não tem; empates pela seed); o capitão é o de maior rating.
    Balanced,
}

#[derive(Deserialize, ToSchema)]
pub struct DrawTeams {
    pub method: DrawMethod,
}

/// Um time novo com um jogador, que vira o capitão.
#[derive(Deserialize, ToSchema)]
pub struct NewTeam {
    /// Um inscrito (sai do time em que estava).
    pub user_id: i64,
}

/// Campos ausentes não mudam.
#[derive(Deserialize, ToSchema)]
pub struct UpdateTeam {
    /// 1 a 40 caracteres, único no torneio sem diferenciar maiúsculas;
    /// `null` (ou vazio) tira o nome (a tela mostra "Time de {capitão}").
    /// O organizador e o capitão, até o torneio encerrar.
    #[serde(default, deserialize_with = "crate::shared::nullable")]
    #[schema(value_type = Option<String>)]
    pub name: Option<Option<String>>,
    /// `user_id` do novo capitão, um jogador do time. Só o organizador, até
    /// o torneio encerrar.
    pub captain: Option<i64>,
}

/// Troca de um jogador do time durante o torneio.
#[derive(Deserialize, ToSchema)]
pub struct Substitution {
    /// Quem sai: um jogador do time (pode ter saído da comunidade). Fica
    /// inscrito, sem time.
    pub out: i64,
    /// Quem entra: um reserva (inscrito sem time) ou outro membro ativo da
    /// comunidade, que fica inscrito já no time.
    #[serde(rename = "in")]
    pub sub: i64,
}

/// `{tournament}`, `{team}` e `{user_id}` de
/// `/tenants/{slug}/tournaments/{tournament}/teams/{team}/members/{user_id}`.
#[derive(Deserialize)]
pub struct TeamMemberPath {
    pub tournament: String,
    pub team: i64,
    pub user_id: i64,
}
