use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{setup::TournamentMode, status::TournamentStatus};
use crate::shared::Locale;

/// Formato de uma fase.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(tag = "format", rename_all = "snake_case")]
pub enum StageFormat {
    Elimination {
        bracket: Bracket,
        /// Só na dupla: se o campeão da chave de baixo vence a grande final,
        /// há uma segunda série. Ignorado na simples.
        #[serde(default)]
        grand_final_reset: bool,
        /// Melhor de N das partidas comuns (ímpar).
        #[schema(example = 3)]
        best_of: u8,
        /// Melhor de N da final (simples) ou da grande final e do reset (dupla).
        #[schema(example = 5)]
        finals_best_of: u8,
    },
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Bracket {
    Single,
    Double,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChampionRules {
    /// Bans de campeão que cada lado sofre por mapa, de 0 a 3.
    #[schema(example = 1)]
    pub bans_per_game: u8,
    /// Regra de não repetição; `null` desliga.
    pub no_repeat: Option<NoRepeatRule>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoRepeatRule {
    pub who: NoRepeatWho,
    pub scope: NoRepeatScope,
}

/// Quem não pode repetir campeão. No duelo os dois dão no mesmo.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NoRepeatWho {
    Player,
    Team,
}

/// Até onde a regra vale.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NoRepeatScope {
    Match,
    Stage,
    Tournament,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateTournament {
    /// a-z, 0-9 e hífen, 3 a 32 caracteres, único no tenant. Não muda depois.
    #[schema(example = "copa-de-outubro")]
    pub slug: String,
    /// Idioma original dos textos. Não muda depois.
    pub locale: Locale,
    /// 1 a 80 caracteres.
    #[schema(example = "Copa de Outubro")]
    pub name: String,
    /// Markdown, até 2000 caracteres.
    #[serde(default)]
    pub description: String,
    /// Markdown, até 20000 caracteres.
    #[serde(default)]
    pub rules: String,
    pub mode: TournamentMode,
    /// Vagas, de 2 a 1024. A checagem de campeões usa o pior caso com esse número.
    #[schema(example = 16)]
    pub max_participants: i32,
    pub champion_rules: ChampionRules,
    /// Fases, na ordem. Por enquanto, exatamente uma.
    pub stages: Vec<StageFormat>,
}

/// Campos ausentes não mudam. Textos podem mudar até o torneio encerrar;
/// vagas, regra de campeões e fases, até ele começar.
#[derive(Deserialize, ToSchema)]
pub struct UpdateTournament {
    pub name: Option<String>,
    pub description: Option<String>,
    pub rules: Option<String>,
    pub max_participants: Option<i32>,
    pub champion_rules: Option<ChampionRules>,
    pub stages: Option<Vec<StageFormat>>,
}

/// Torneio completo, com os textos no idioma original e as traduções; o
/// front escolhe pelo idioma de quem vê.
#[derive(Serialize, ToSchema)]
pub struct Tournament {
    pub slug: String,
    pub locale: Locale,
    pub name: String,
    pub description: String,
    pub rules: String,
    pub mode: TournamentMode,
    pub status: TournamentStatus,
    pub max_participants: i32,
    pub champion_rules: ChampionRules,
    pub stages: Vec<StageFormat>,
    pub translations: Vec<TournamentTranslation>,
    pub created_at: DateTime<Utc>,
}

/// Item da lista de torneios do tenant.
#[derive(Serialize, ToSchema)]
pub struct TournamentSummary {
    pub slug: String,
    pub locale: Locale,
    pub name: String,
    pub mode: TournamentMode,
    pub status: TournamentStatus,
    pub max_participants: i32,
    /// Nome nas traduções que existem.
    pub translations: Vec<NameTranslation>,
    pub created_at: DateTime<Utc>,
}

/// Tradução de um torneio. Descrição ou regras vazias: o front mostra as do original.
#[derive(Serialize, ToSchema)]
pub struct TournamentTranslation {
    pub locale: Locale,
    pub name: String,
    pub description: String,
    pub rules: String,
}

#[derive(Serialize, ToSchema)]
pub struct NameTranslation {
    pub locale: Locale,
    pub name: String,
}

#[derive(Deserialize, ToSchema)]
pub struct PutTranslation {
    /// 1 a 80 caracteres.
    pub name: String,
    /// Markdown, até 2000 caracteres; vazio usa o original.
    #[serde(default)]
    pub description: String,
    /// Markdown, até 20000 caracteres; vazio usa o original.
    #[serde(default)]
    pub rules: String,
}

/// `{tournament}` de `/tenants/{slug}/tournaments/{tournament}` (o slug do
/// tenant fica com `TenantMember`).
#[derive(Deserialize)]
pub struct TournamentPath {
    pub tournament: String,
}

#[derive(Deserialize)]
pub struct TranslationPath {
    pub tournament: String,
    pub locale: Locale,
}
