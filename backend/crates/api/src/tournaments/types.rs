use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{
    admission::Admission,
    contest::DEFAULT_CONTEST_MINUTES,
    schedule::DEFAULT_CHECK_IN_MINUTES,
    setup::TournamentMode,
    status::TournamentStatus,
    team_rules::TeamRules,
    veto::{VetoInput, VetoSettings},
};
use crate::shared::{Locale, nullable};

/// Formato de uma fase.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, PartialEq, Eq)]
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
        /// Melhor de N de todas as partidas, ímpar ou par. A série par
        /// fecha na maioria ou, com todos os mapas e metade para cada lado, empata.
        #[schema(example = 3)]
        best_of: u8,
        /// Os desempates depois do saldo de séries, na ordem em que valem; por
        /// último, sempre a ordem da chave. Cada um no máximo uma vez, sem
        /// `buchholz`; vazio: decide a ordem da chave. Ausente: `head_to_head`,
        /// `map_diff`, `frag_diff`.
        #[serde(default = "round_robin_tiebreakers")]
        #[schema(example = json!(["head_to_head", "map_diff", "frag_diff"]))]
        tiebreakers: Vec<Tiebreaker>,
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
        /// Os desempates depois dos pontos, na ordem em que valem; por último,
        /// sempre a ordem da chave. Cada um no máximo uma vez; vazio: decide a
        /// ordem da chave. O pareamento segue a tabela, então eles mudam
        /// também quem enfrenta quem. Ausente: `buchholz`, `head_to_head`,
        /// `map_diff`, `frag_diff`.
        #[serde(default = "swiss_tiebreakers")]
        #[schema(example = json!(["buchholz", "head_to_head", "map_diff", "frag_diff"]))]
        tiebreakers: Vec<Tiebreaker>,
    },
    /// Disputa de posições, a fase inteira: a ordem da chave forma os pares
    /// (1 x 2, 3 x 4...), todos prontos no início, e quem vence cada série
    /// fica com a melhor das duas posições. Com N ímpar, o último fica na
    /// posição dele sem jogar. De 2 a 128 participantes.
    Placement {
        /// Melhor de N de todas as partidas (ímpar).
        #[schema(example = 3)]
        best_of: u8,
    },
}

/// Um desempate das fases de tabela. O primeiro critério (saldo de séries ou
/// pontos) e a ordem da chave, o último, são fixos e não entram na lista.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Tiebreaker {
    /// Confronto direto: as séries entre os empatados, quando todos já se
    /// enfrentaram.
    HeadToHead,
    /// Saldo de mapas.
    MapDiff,
    /// Saldo de frags, quando todos os empatados têm os frags de todas as
    /// séries.
    FragDiff,
    /// Soma dos pontos dos adversários. Só no suíço.
    Buchholz,
}

fn round_robin_tiebreakers() -> Vec<Tiebreaker> {
    Tiebreaker::list(engine::formats::table::Tiebreakers::ROUND_ROBIN)
}

fn swiss_tiebreakers() -> Vec<Tiebreaker> {
    Tiebreaker::list(engine::formats::table::Tiebreakers::SWISS)
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Bracket {
    Single,
    Double,
}

/// As regras do draft de campeões, feito para a série inteira antes da
/// partida 1.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChampionRules {
    /// Bans de campeão de cada lado na série, de 0 a 3; valem para os dois
    /// lados em todas as partidas.
    #[schema(example = 1)]
    pub bans_per_series: u8,
    /// Adversários podem usar o mesmo campeão na mesma partida. Sem isso,
    /// fica com ele quem escolhe primeiro.
    #[schema(example = true)]
    pub mirror: bool,
    /// TDM: colegas de time podem usar o mesmo campeão na mesma partida. No
    /// duelo, não se aplica.
    pub same_team: bool,
    /// Não repetir o campeão de outra partida; `null`: pode repetir.
    pub no_repeat: Option<NoRepeatRule>,
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

/// Até onde a regra vale: entre as partidas da série, em todas as séries da
/// fase ou no torneio inteiro. Fora da série, conta só partida jogada.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NoRepeatScope {
    Series,
    Stage,
    Tournament,
}

/// As regras de campeões da criação: o que faltar cai no padrão
/// (`ChampionRulesInput::default`). O PATCH segue com `ChampionRules`
/// inteira.
#[derive(Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChampionRulesInput {
    /// Ausente: 1.
    #[serde(default = "default_bans_per_series")]
    #[schema(required = false, example = 1)]
    pub bans_per_series: u8,
    /// Ausente: verdadeiro.
    #[serde(default = "default_mirror")]
    #[schema(required = false)]
    pub mirror: bool,
    /// Padrão falso.
    #[serde(default)]
    pub same_team: bool,
    /// Padrão `null`: pode repetir.
    #[serde(default)]
    pub no_repeat: Option<NoRepeatRule>,
}

impl Default for ChampionRulesInput {
    /// As regras de um torneio novo.
    fn default() -> Self {
        Self {
            bans_per_series: default_bans_per_series(),
            mirror: default_mirror(),
            same_team: false,
            no_repeat: None,
        }
    }
}

impl From<ChampionRulesInput> for ChampionRules {
    fn from(input: ChampionRulesInput) -> Self {
        Self {
            bans_per_series: input.bans_per_series,
            mirror: input.mirror,
            same_team: input.same_team,
            no_repeat: input.no_repeat,
        }
    }
}

/// Vagas de um torneio novo.
const DEFAULT_MAX_PARTICIPANTS: i32 = 16;
/// Bans por série de um torneio novo.
const DEFAULT_BANS_PER_SERIES: u8 = 1;
/// Melhor de N das séries comuns de um torneio novo.
const DEFAULT_BEST_OF: u8 = 3;
/// Melhor de N da final (e da grande final) de um torneio novo.
const DEFAULT_FINALS_BEST_OF: u8 = 5;

fn default_max_participants() -> i32 {
    DEFAULT_MAX_PARTICIPANTS
}

fn default_bans_per_series() -> u8 {
    DEFAULT_BANS_PER_SERIES
}

fn default_mirror() -> bool {
    true
}

/// A fase de um torneio novo: eliminação dupla, md3, final md5, sem reset.
fn default_stages() -> Vec<StageFormat> {
    vec![StageFormat::Elimination {
        bracket: Bracket::Double,
        grand_final_reset: false,
        best_of: DEFAULT_BEST_OF,
        finals_best_of: DEFAULT_FINALS_BEST_OF,
    }]
}

#[derive(Deserialize, ToSchema)]
pub struct CreateTournament {
    /// a-z, 0-9 e hífen, 3 a 32 caracteres, único no tenant. Muda enquanto for
    /// rascunho.
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
    /// times que ele comporta). Ausente: 16.
    #[serde(default = "default_max_participants")]
    #[schema(required = false, example = 16)]
    pub max_participants: i32,
    /// Ausente, ou campo ausente: 1 ban por série, espelho permitido, sem
    /// campeão igual no time e sem regra de não repetição.
    #[serde(default)]
    #[schema(required = false)]
    pub champion_rules: ChampionRulesInput,
    /// Fases, na ordem. Por enquanto, exatamente uma. Ausente: uma fase de
    /// eliminação dupla, melhor de 3, final melhor de 5, sem reset da grande
    /// final.
    #[serde(default = "default_stages")]
    #[schema(required = false)]
    pub stages: Vec<StageFormat>,
    /// Início previsto. Opcional: sem data o check-in não abre sozinho.
    #[serde(default)]
    pub starts_at: Option<DateTime<Utc>>,
    /// Minutos antes do início em que o check-in abre sozinho, de 5 a 1440.
    /// Ausente: 30. `null`: sem check-in (a inscrição vai direto para em
    /// andamento).
    #[serde(default = "default_check_in_minutes")]
    #[schema(required = false, example = 30)]
    pub check_in_minutes: Option<i32>,
    /// Minutos que o adversário tem para confirmar ou contestar um resultado
    /// enviado, de 1 a 1440; depois disso, confirma sozinho. Ausente: 10.
    #[serde(default = "default_contest_minutes")]
    #[schema(required = false, example = 10)]
    pub contest_minutes: i32,
    /// Veto das partidas: modo do mapa, pool, campeões e tempo por passo.
    /// Ausente (ou campo ausente): o padrão (veto, os 7 primeiros mapas do
    /// pool de duelo, campeões ligados, 30 s).
    #[serde(default)]
    pub veto: Option<VetoInput>,
    /// Quem age pelo time. Ausente: o capitão em tudo (`TeamRules::CAPTAIN`).
    /// No duelo, não muda nada.
    #[serde(default)]
    pub team_rules: Option<TeamRules>,
    /// Traduções dos textos, no máximo uma por idioma e nenhuma no idioma
    /// original. Erro de texto numa delas (`tournament.name_invalid`,
    /// `tournament.description_invalid`, `tournament.rules_invalid`) e
    /// `tournament.translation_same_locale` trazem `params.locale` com o
    /// idioma dela; idioma repetido dá `request.invalid_body`.
    #[serde(default)]
    pub translations: Vec<CreateTranslation>,
    /// Padrão falso (rascunho). Verdadeiro cria e já publica, na mesma
    /// transação: o torneio vem sempre em `registration` (inscrição aberta);
    /// o check-in abre depois, na hora (um `starts_at` dentro da janela já
    /// aberta é recusado com `tournament.starts_at_too_soon`). Qualquer erro e
    /// nada é gravado.
    #[serde(default)]
    pub publish: bool,
    /// Se os mapas do torneio contam para o rating da comunidade. Ausente:
    /// sim. Muda até o torneio começar.
    #[serde(default = "default_counts_for_rating")]
    #[schema(required = false)]
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

/// Campos ausentes, ou iguais aos gravados, não mudam. O endereço muda só no
/// rascunho; nome e descrição, sempre; regras e inscrição, até o torneio
/// encerrar; vagas, regra de campeões, fases, data de início e janela de
/// check-in, até ele começar (e as vagas nunca abaixo do número de inscritos).
#[derive(Deserialize, ToSchema)]
pub struct UpdateTournament {
    /// O endereço novo, só no rascunho, com a validação da criação. O antigo
    /// fica livre na hora, sem redirecionamento; links de inscrição continuam
    /// valendo (usam o token).
    #[schema(example = "copa-de-novembro")]
    pub slug: Option<String>,
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
    /// Quem age pelo time, até o torneio começar; ausente não muda.
    #[serde(default)]
    pub team_rules: Option<TeamRules>,
    /// Screenshot obrigatório; muda até o torneio encerrar e vale para os
    /// envios seguintes. Ligar exige o bucket configurado.
    pub screenshots_required: Option<bool>,
}

/// Corpo de `DELETE /tenants/{slug}/tournaments/{tournament}`: o rascunho não
/// precisa dele.
#[derive(Deserialize, ToSchema)]
pub struct DeleteTournament {
    /// O slug do torneio, digitado para confirmar.
    #[schema(example = "copa-de-outubro")]
    pub confirm_slug: String,
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
    /// Quem age pelo time.
    pub team_rules: TeamRules,
    /// O envio do jogador exige screenshot do placar (sem o bucket configurado,
    /// fica gravado e não vale).
    pub screenshots_required: bool,
    pub translations: Vec<TournamentTranslation>,
    pub created_at: DateTime<Utc>,
    /// A capa (2:1), no CDN. Nulo: sem capa, ou a API sem o bucket público.
    pub cover_url: Option<String>,
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
    /// A capa (2:1), no CDN. Nulo: sem capa, ou a API sem o bucket público.
    pub cover_url: Option<String>,
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
