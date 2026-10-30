use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{
    admission::Admission,
    contest::DEFAULT_CONTEST_MINUTES,
    schedule::DEFAULT_CHECK_IN_MINUTES,
    setup::TournamentMode,
    status::TournamentStatus,
    veto::{VetoInput, VetoSettings},
};
use crate::shared::{Locale, nullable};

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
    /// Round robin simples, a fase inteira: cada par se enfrenta uma vez e a
    /// classificação sai da tabela. Até 32 participantes.
    RoundRobin {
        /// Melhor de N de todas as partidas (ímpar).
        #[schema(example = 3)]
        best_of: u8,
    },
    /// Suíço, a fase inteira: rodadas fixas, pareamento por pontos sem
    /// revanche, classificação pela tabela. Até 128 participantes.
    Swiss {
        /// Melhor de N de todas as partidas (ímpar).
        #[schema(example = 3)]
        best_of: u8,
        /// Rodadas, de 1 a 127; `null` (ou ausente): ⌈log2 N⌉. Na fase, no
        /// máximo N − 1 com os N do início.
        #[serde(default)]
        rounds: Option<u16>,
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
    /// TDM: dois jogadores do mesmo time podem escolher o mesmo campeão no
    /// mesmo mapa. Ausente: não (entre times, o espelho vale sempre).
    #[serde(default)]
    pub team_duplicates: bool,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoRepeatRule {
    pub who: NoRepeatWho,
    pub scope: NoRepeatScope,
}

/// Quem não pode repetir campeão: cada jogador os próprios, ou o time
/// inteiro (o que um usou sai de todos). No duelo os dois dão no mesmo.
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
    /// Texto simples; as quebras de linha são preservadas. Até 2000 caracteres.
    #[serde(default)]
    pub description: String,
    /// Texto simples; as quebras de linha são preservadas. Até 20000 caracteres.
    #[serde(default)]
    pub rules: String,
    /// Duelo ou TDM. Muda até o torneio começar; com inscritos, refaz os
    /// times (no duelo, cada um num time de um; no TDM, todos sem time).
    pub mode: TournamentMode,
    /// Jogadores por time: 1 no duelo; de 2 a 4 no TDM. Ausente: 1 no duelo,
    /// 2 no TDM.
    #[serde(default)]
    pub team_size: Option<i32>,
    /// Padrão `open`. Muda depois, até o torneio encerrar.
    #[serde(default)]
    pub admission: Admission,
    /// Vagas, em jogadores, de 2 a 1024 (no TDM, no mínimo dois times). A
    /// checagem de campeões usa o pior caso com esse número (no TDM, com os
    /// times que ele comporta).
    #[schema(example = 16)]
    pub max_participants: i32,
    pub champion_rules: ChampionRules,
    /// Fases, na ordem. Por enquanto, exatamente uma.
    pub stages: Vec<StageFormat>,
    /// Início previsto. Opcional: sem data o check-in não abre sozinho.
    #[serde(default)]
    pub starts_at: Option<DateTime<Utc>>,
    /// Minutos antes do início em que o check-in abre sozinho, de 5 a 1440.
    /// Ausente: 30. `null`: sem check-in (a inscrição vai direto para em
    /// andamento).
    #[serde(default = "default_check_in_minutes")]
    #[schema(required = false, default = 30, example = 30)]
    pub check_in_minutes: Option<i32>,
    /// Minutos que o adversário tem para confirmar ou contestar um resultado
    /// enviado, de 1 a 1440; depois disso, confirma sozinho. Ausente: 5.
    #[serde(default = "default_contest_minutes")]
    #[schema(required = false, default = 5, example = 5)]
    pub contest_minutes: i32,
    /// Veto das partidas: modo do mapa, pool, campeões e tempo por passo.
    /// Ausente (ou campo ausente): o padrão (veto, os 7 primeiros mapas do
    /// pool de duelo, campeões ligados, 30 s).
    #[serde(default)]
    pub veto: Option<VetoInput>,
    /// Traduções dos textos, no máximo uma por idioma e nenhuma no idioma
    /// original. Erro de texto numa delas (`tournament.name_invalid`,
    /// `tournament.description_invalid`, `tournament.rules_invalid`) e
    /// `tournament.translation_same_locale` trazem `params.locale` com o
    /// idioma dela; idioma repetido dá `request.invalid_body`.
    #[serde(default)]
    pub translations: Vec<CreateTranslation>,
    /// Padrão falso (rascunho). Verdadeiro cria e já publica, na mesma
    /// transação: e, se há `starts_at` e a
    /// janela de check-in já abriu, o torneio vem em `check_in`; senão, em
    /// `registration`. Qualquer erro e nada é gravado.
    #[serde(default)]
    pub publish: bool,
    /// Se os mapas do torneio contam para o rating da comunidade. Ausente:
    /// sim. Muda até o torneio começar.
    #[serde(default = "default_counts_for_rating")]
    #[schema(required = false, default = true)]
    pub counts_for_rating: bool,
    /// O envio do jogador exige screenshot do placar (`screenshots`). Só liga
    /// com o bucket configurado (`GET /screenshots/settings`). Ausente:
    /// `false`.
    #[serde(default)]
    pub screenshots_required: bool,
}

/// Tradução enviada junto com a criação do torneio.
#[derive(Deserialize, ToSchema)]
pub struct CreateTranslation {
    pub locale: Locale,
    /// 1 a 80 caracteres.
    pub name: String,
    /// Texto simples; as quebras de linha são preservadas. Até 2000
    /// caracteres; vazio usa o original.
    #[serde(default)]
    pub description: String,
    /// Texto simples; as quebras de linha são preservadas. Até 20000
    /// caracteres; vazio usa o original.
    #[serde(default)]
    pub rules: String,
}

fn default_check_in_minutes() -> Option<i32> {
    Some(DEFAULT_CHECK_IN_MINUTES)
}

fn default_contest_minutes() -> i32 {
    DEFAULT_CONTEST_MINUTES
}

fn default_counts_for_rating() -> bool {
    true
}

/// Campos ausentes não mudam. Textos e inscrição podem mudar até o
/// torneio encerrar; vagas, regra de campeões, fases, data de início e
/// janela de check-in, até ele começar (e as vagas nunca abaixo do número de
/// inscritos).
#[derive(Deserialize, ToSchema)]
pub struct UpdateTournament {
    /// Até o torneio começar. Trocar de modo (ou de tamanho) com inscritos
    /// refaz os times na mesma transação: no duelo, cada inscrito num time
    /// de um; no TDM, todos sem time (a formação começa de novo).
    pub mode: Option<TournamentMode>,
    /// Ausente com o modo trocado: o padrão do modo novo (1 no duelo, 2 no
    /// TDM).
    pub team_size: Option<i32>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub rules: Option<String>,
    pub admission: Option<Admission>,
    pub max_participants: Option<i32>,
    pub champion_rules: Option<ChampionRules>,
    pub stages: Option<Vec<StageFormat>>,
    /// `null` tira a data, o que só vale no rascunho.
    #[serde(default, deserialize_with = "nullable")]
    #[schema(value_type = Option<DateTime<Utc>>)]
    pub starts_at: Option<Option<DateTime<Utc>>>,
    /// `null` desliga o check-in; durante o check-in, o torneio volta para a
    /// inscrição.
    #[serde(default, deserialize_with = "nullable")]
    #[schema(value_type = Option<i32>)]
    pub check_in_minutes: Option<Option<i32>>,
    /// Janela de contestação; muda até o torneio encerrar e vale para os
    /// resultados enviados depois.
    pub contest_minutes: Option<i32>,
    /// Se conta para o rating; muda até o torneio começar.
    pub counts_for_rating: Option<bool>,
    /// Veto das partidas, até o torneio começar; campo ausente não muda.
    #[serde(default)]
    pub veto: Option<VetoInput>,
    /// Screenshot obrigatório; muda até o torneio encerrar e vale para os
    /// envios seguintes. Ligar exige o bucket configurado.
    pub screenshots_required: Option<bool>,
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
    /// Jogadores por time (1 no duelo).
    pub team_size: i32,
    pub status: TournamentStatus,
    pub admission: Admission,
    /// Vagas, em jogadores.
    pub max_participants: i32,
    /// Inscritos agora.
    pub participants: i64,
    /// Se quem vê está inscrito.
    pub registered: bool,
    /// Se quem vê fez check-in.
    pub checked_in: bool,
    pub champion_rules: ChampionRules,
    pub stages: Vec<StageFormat>,
    /// Início previsto; `null`: sem data.
    pub starts_at: Option<DateTime<Utc>>,
    /// Janela de check-in em minutos; `null`: sem check-in.
    pub check_in_minutes: Option<i32>,
    /// Quando o check-in abre (ou abriu) sozinho: o início menos a janela.
    pub check_in_opens_at: Option<DateTime<Utc>>,
    /// Minutos que o adversário tem para confirmar ou contestar um resultado
    /// enviado.
    pub contest_minutes: i32,
    /// Se os mapas do torneio contam para o rating da comunidade.
    pub counts_for_rating: bool,
    pub veto: VetoSettings,
    /// O envio do jogador exige screenshot do placar (sem o bucket configurado,
    /// fica gravado e não vale).
    pub screenshots_required: bool,
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
    /// Jogadores por time (1 no duelo).
    pub team_size: i32,
    pub status: TournamentStatus,
    pub admission: Admission,
    pub max_participants: i32,
    /// Inscritos agora.
    pub participants: i64,
    /// Se quem vê está inscrito.
    pub registered: bool,
    /// Se quem vê fez check-in (no check-in, falso é o aviso de que falta).
    pub checked_in: bool,
    /// Início previsto; `null`: sem data.
    pub starts_at: Option<DateTime<Utc>>,
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
    /// Texto simples; as quebras de linha são preservadas. Até 2000 caracteres; vazio usa o original.
    #[serde(default)]
    pub description: String,
    /// Texto simples; as quebras de linha são preservadas. Até 20000 caracteres; vazio usa o original.
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
