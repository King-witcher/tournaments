use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{shared::Locale, teams::Team, tournaments::NameTranslation};

/// O ranking da comunidade.
#[derive(Serialize, ToSchema)]
pub struct Ranking {
    /// Rating de quem ainda não jogou.
    #[schema(example = 1500)]
    pub initial: i32,
    /// Com menos jogos que isto, o rating é provisório.
    #[schema(example = 10)]
    pub provisional_games: u32,
    /// Os membros ativos com rating (algum mapa ou ajuste), do maior para o
    /// menor; no empate, quem jogou mais e depois o apelido.
    pub players: Vec<RankingEntry>,
}

/// Uma pessoa no ranking.
#[derive(Serialize, ToSchema, Clone)]
pub struct RankingEntry {
    /// 1 é o maior rating; quem empata no rating divide a posição (1, 2, 2, 4).
    pub position: i64,
    pub user_id: i64,
    pub nickname: String,
    /// Nome exibido do usuário: o do Discord ou o username do cadastro.
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub rating: i32,
    /// Mapas que contaram.
    pub games: i32,
    pub wins: i32,
    pub losses: i32,
    /// Quando confirmou o último mapa que contou; `null` sem mapa (só ajustes).
    pub last_game_at: Option<DateTime<Utc>>,
    /// Menos de `provisional_games` mapas.
    pub provisional: bool,
}

/// O rating de um membro, com os últimos mapas que contaram.
#[derive(Serialize, ToSchema)]
pub struct MemberRating {
    pub user_id: i64,
    /// `null` enquanto não tem nenhum mapa nem ajuste (o rating seria o
    /// `initial`).
    pub standing: Option<RankingEntry>,
    pub initial: i32,
    pub provisional_games: u32,
    /// Soma dos ajustes manuais da organização, já dentro do rating.
    pub adjusted: i32,
    /// Os últimos mapas que contaram, do mais recente ao mais antigo, só dos
    /// torneios que quem vê vê na lista da comunidade.
    pub games: Vec<RatedGame>,
    /// Os ajustes manuais, do mais recente ao mais antigo; só para organizer
    /// e owner (`null` para player).
    pub adjustments: Option<Vec<RatingAdjustment>>,
}

/// Um mapa que contou para o rating de alguém.
#[derive(Serialize, ToSchema)]
pub struct RatedGame {
    pub tournament: RatedTournament,
    /// A partida (`wb-1-3`).
    #[serde(rename = "match")]
    pub match_key: String,
    /// Número do mapa na série.
    pub game: i16,
    /// Nome do mapa; `null` nas séries antigas enviadas sem o detalhe.
    pub map: Option<String>,
    /// O time do outro lado do mapa (no duelo, o do adversário).
    pub opponent: Team,
    pub won: bool,
    /// Pontos ganhos (positivo) ou perdidos (negativo) neste mapa.
    pub delta: i32,
    /// O rating depois dele.
    pub rating_after: i32,
    /// Quando o mapa confirmou.
    pub at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct RatedTournament {
    pub slug: String,
    pub locale: Locale,
    /// No idioma original; as traduções ao lado.
    pub name: String,
    pub translations: Vec<NameTranslation>,
}

/// Um ajuste manual.
#[derive(Serialize, ToSchema)]
pub struct RatingAdjustment {
    pub id: i64,
    pub delta: i32,
    pub reason: String,
    /// `null` se quem ajustou foi apagado.
    pub created_by: Option<AdjustmentAuthor>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct AdjustmentAuthor {
    pub user_id: i64,
    pub nickname: String,
}

#[derive(Deserialize, ToSchema)]
pub struct NewAdjustment {
    /// Pontos a somar (negativo tira), de -1000 a 1000, sem zero.
    #[schema(example = 50)]
    pub delta: i32,
    /// Motivo, de 1 a 280 caracteres; espaços nas pontas são removidos.
    #[schema(example = "Rating de outra liga")]
    pub reason: String,
}

/// Os parâmetros do Elo da comunidade. Mudar recalcula todos os ratings com
/// o histórico inteiro.
#[derive(Serialize, Deserialize, ToSchema)]
pub struct RatingSettings {
    /// Rating de quem ainda não jogou, de 100 a 3000.
    #[schema(example = 1500)]
    pub initial: i32,
    /// K de quem já jogou muito, de 1 a 100.
    #[schema(example = 32)]
    pub k_min: i32,
    /// K do primeiro mapa, de 1 a 200, não abaixo de `k_min`.
    #[schema(example = 80)]
    pub k_max: i32,
    /// Em quantos mapas a distância do K até o mínimo cai para ~37%, de 1 a
    /// 100: K = k_min + (k_max - k_min) · e^(-mapas / k_decay).
    #[schema(example = 10)]
    pub k_decay: i32,
}

/// `{user_id}` de `/tenants/{slug}/ratings/{user_id}/...` (o slug fica com `TenantMember`).
#[derive(Deserialize)]
pub struct RatingPath {
    pub user_id: i64,
}
