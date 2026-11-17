use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    my_tournaments::{MyPlacement, Situation},
    shared::Locale,
    teams::Team,
    tenants::Role,
    tournaments::{NameTranslation, TournamentMode, TournamentSummary},
};

#[derive(Serialize, ToSchema)]
pub struct Member {
    pub user_id: i64,
    pub nickname: String,
    /// Nome exibido do usuário: o do Discord ou o do cadastro.
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub role: Role,
    pub joined_at: DateTime<Utc>,
}

/// Um torneio em que o membro se inscreveu.
#[derive(Serialize, ToSchema)]
pub struct MemberTournament {
    /// Como na lista da comunidade: `registered` e `checked_in` são de quem
    /// vê, não do membro.
    pub tournament: TournamentSummary,
    /// A situação do membro no torneio.
    pub situation: Situation,
    /// Colocação do membro, se foi eliminado, desistiu ou venceu, quando já
    /// se definiu.
    pub placement: Option<MyPlacement>,
    /// TDM: o time do membro (`null` no duelo e sem time).
    pub team: Option<Team>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateMyMembership {
    /// 1 a 32 caracteres; espaços nas pontas são removidos.
    pub nickname: String,
}

/// Papéis que o owner pode dar. A posse do tenant não muda por aqui.
#[derive(Deserialize, ToSchema, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum AssignableRole {
    Organizer,
    Player,
}

impl From<AssignableRole> for Role {
    fn from(role: AssignableRole) -> Self {
        match role {
            AssignableRole::Organizer => Self::Organizer,
            AssignableRole::Player => Self::Player,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct SetRole {
    pub role: AssignableRole,
}

/// `{user_id}` de `/tenants/{slug}/members/{user_id}/...` (o slug fica com `TenantMember`).
#[derive(Deserialize)]
pub struct MemberPath {
    pub user_id: i64,
}

/// O nome de um torneio, para mostrar no idioma de quem vê.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, PartialEq, Eq)]
pub struct TournamentLabel {
    pub slug: String,
    pub locale: Locale,
    /// No idioma original; as traduções ao lado.
    pub name: String,
    pub translations: Vec<NameTranslation>,
}

/// Vitórias, derrotas e empates em séries (empate só na série de número par
/// de mapas do todos contra todos).
#[derive(Serialize, ToSchema, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SeriesRecord {
    pub wins: i32,
    pub losses: i32,
    pub draws: i32,
}

/// Vitórias e derrotas em mapas.
#[derive(Serialize, ToSchema, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GameRecord {
    pub wins: i32,
    pub losses: i32,
}

/// A sequência atual: as últimas séries decididas com o mesmo resultado.
#[derive(Serialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Streak {
    /// `true`: vitórias; `false`: derrotas.
    pub won: bool,
    /// Quantas seguidas, de 1 em diante.
    pub count: i32,
}

#[derive(Serialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SeriesOutcome {
    Won,
    Lost,
    Draw,
}

/// A carreira de um membro na comunidade: as séries decididas e os mapas
/// confirmados de todos os torneios dela, com rating ligado ou não.
#[derive(Serialize, ToSchema)]
pub struct MemberCareer {
    /// As séries decididas, W.O. incluído.
    pub series: SeriesRecord,
    /// Os mapas confirmados.
    pub games: GameRecord,
    /// Frags feitos nos mapas com placar.
    pub frags_for: i32,
    /// Frags sofridos nos mapas com placar.
    pub frags_against: i32,
    /// Torneios vencidos.
    pub titles: i32,
    /// Torneios terminados entre os três primeiros (os títulos incluídos).
    pub podiums: i32,
    /// A melhor colocação já definida; `null` sem nenhuma.
    pub best_placement: Option<MyPlacement>,
    /// `null` sem série decidida ou com a última empatada.
    pub streak: Option<Streak>,
    /// As últimas séries decididas, da mais recente.
    pub recent_series: Vec<MemberSeries>,
}

/// Uma série decidida de um membro.
#[derive(Serialize, ToSchema)]
pub struct MemberSeries {
    pub tournament: TournamentLabel,
    /// A partida, pelo código público (`wb-1-3`; `s2-wb-1-3` da fase 2 em
    /// diante).
    #[serde(rename = "match")]
    pub match_key: String,
    /// O time do outro lado (no duelo, o do adversário).
    pub opponent: Team,
    pub outcome: SeriesOutcome,
    /// Mapas vencidos pelo lado do membro; `null` no W.O.
    pub wins: Option<i16>,
    /// Mapas vencidos pelo outro lado; `null` no W.O.
    pub losses: Option<i16>,
    /// Quando a série foi decidida.
    pub at: DateTime<Utc>,
}

/// Filtros de `GET /tenants/{slug}/members/{user_id}/games`.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GamesQuery {
    /// Página, de 1 em diante. Sem ela, 1.
    #[param(minimum = 1)]
    pub page: Option<i64>,
    /// Só os mapas deste torneio (slug).
    pub tournament: Option<String>,
    /// Só os mapas desta arena (slug).
    pub map: Option<String>,
    /// Só os mapas em que esta pessoa (`user_id`) estava do outro lado.
    pub opponent: Option<i64>,
}

/// Uma página dos mapas de um membro.
#[derive(Serialize, ToSchema)]
pub struct MemberGames {
    /// Quantos mapas o filtro acha, em todas as páginas.
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    /// Do mais recente ao mais antigo.
    pub games: Vec<MemberGame>,
    /// As opções de cada filtro, de todos os mapas do membro (não só os do
    /// filtro atual).
    pub filters: GameFilters,
}

/// Um mapa confirmado de um membro.
#[derive(Serialize, ToSchema)]
pub struct MemberGame {
    pub tournament: TournamentLabel,
    /// A partida, pelo código público (`wb-1-3`; `s2-wb-1-3` da fase 2 em
    /// diante).
    #[serde(rename = "match")]
    pub match_key: String,
    /// A fase da partida é eliminação simples (o nome curto dela é outro:
    /// "Final", não "C3.1").
    pub single: bool,
    /// Número do mapa na série.
    pub game: i16,
    /// A arena; `null` nas séries antigas enviadas sem o detalhe.
    pub map: Option<MapLabel>,
    /// O time do outro lado do mapa (no duelo, o do adversário).
    pub opponent: Team,
    pub won: bool,
    /// Frags do lado do membro; `null` sem placar.
    pub frags: Option<i16>,
    /// Frags do outro lado; `null` sem placar.
    pub opponent_frags: Option<i16>,
    /// O efeito no rating; `null` se o mapa não contou (torneio fora do
    /// rating).
    pub rating: Option<GameRating>,
    /// Quando o mapa confirmou.
    pub at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, PartialEq, Eq)]
pub struct MapLabel {
    #[schema(example = "vestibule-of-exile")]
    pub slug: String,
    /// Nome próprio do mapa, igual em todos os idiomas.
    pub name: String,
}

/// Os pontos que um mapa deu ou tirou do rating.
#[derive(Serialize, ToSchema)]
pub struct GameRating {
    /// Ganhos (positivo) ou perdidos (negativo).
    pub delta: i32,
    /// O rating depois do mapa.
    pub rating_after: i32,
}

/// As opções dos filtros de partidas.
#[derive(Serialize, ToSchema)]
pub struct GameFilters {
    /// Os torneios em que o membro jogou, do mapa mais recente.
    pub tournaments: Vec<TournamentLabel>,
    /// As arenas que ele jogou, por nome.
    pub maps: Vec<MapLabel>,
    /// Quem esteve do outro lado em algum mapa, por apelido.
    pub opponents: Vec<PersonLabel>,
}

/// Uma pessoa da comunidade, ativa ou não.
#[derive(Serialize, ToSchema, Debug, Clone, PartialEq, Eq)]
pub struct PersonLabel {
    pub user_id: i64,
    /// Vazio: conta excluída.
    pub nickname: String,
    pub avatar_url: Option<String>,
    /// Ainda é membro (quem saiu não tem página).
    pub active: bool,
}

/// Filtro de `GET /tenants/{slug}/members/{user_id}/opponents`.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct OpponentsQuery {
    /// Só os torneios desta modalidade. Sem ele, todos.
    pub modality: Option<TournamentMode>,
}

/// O confronto direto de um membro com cada pessoa que esteve do outro lado.
#[derive(Serialize, ToSchema)]
pub struct MemberOpponents {
    /// Da pessoa com mais séries contra o membro à com menos (no empate, mais
    /// mapas, depois o apelido).
    pub opponents: Vec<OpponentRecord>,
}

/// O confronto de um membro com uma pessoa, do ponto de vista do membro.
#[derive(Serialize, ToSchema, Debug, PartialEq, Eq)]
pub struct OpponentRecord {
    pub person: PersonLabel,
    /// As séries decididas em lados opostos.
    pub series: SeriesRecord,
    /// Os mapas confirmados em lados opostos.
    pub games: GameRecord,
    /// O último mapa ou série entre os dois.
    pub last_at: DateTime<Utc>,
}
