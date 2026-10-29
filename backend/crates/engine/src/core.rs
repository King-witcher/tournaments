//! Tipos compartilhados pelos formatos.

use serde::{Deserialize, Serialize};

/// Quem ocupa um lado da chave: um time, em todos os modos (no duelo, um time
/// de um jogador). É o id interno do time (`teams.id` na API, bigint).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ParticipantId(pub i64);

/// Uma pessoa: o usuário (`users.id` na API). Joga por um time (no duelo, o
/// time de um jogador), tem rating e, no TDM, escolhe o próprio campeão.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PlayerId(pub i64);

/// Mapa do jogo: o id interno da tabela de mapas (`maps.id` na API).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MapId(pub i64);

/// Campeão do jogo: o id interno da tabela de campeões (`champions.id` na
/// API).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ChampionId(pub i64);

/// Em qual parte da chave a partida está.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BracketSide {
    /// Chave de cima (ou a única, na eliminação simples).
    Winners,
    /// Chave de baixo da eliminação dupla.
    Losers,
    /// Grande final da eliminação dupla: rodada 1 é a final, rodada 2 é o reset.
    GrandFinal,
    /// Rodada do round robin.
    RoundRobin,
    /// Rodada do suíço.
    Swiss,
}

/// Identificador estável e legível de uma partida ("chave de cima, rodada 2,
/// partida 3"; "suíço, rodada 4, partida 1"). Rodada e índice começam em 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MatchId {
    pub side: BracketSide,
    pub round: u16,
    pub index: u16,
}

/// De onde vem quem ocupa um lado da partida.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Slot {
    Participant(ParticipantId),
    WinnerOf(MatchId),
    LoserOf(MatchId),
    /// Ninguém ocupa este lado: o outro avança sem jogar.
    Bye,
}

/// Quem ocupa um lado de uma partida agora, conforme os resultados.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Occupant {
    /// Ainda depende de uma partida sem resultado.
    Unknown,
    Participant(ParticipantId),
    /// Ninguém: o outro lado avança sem jogar.
    Bye,
}

/// Placar de uma série melhor de N, em mapas vencidos.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeriesScore {
    pub winner: u8,
    pub loser: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Outcome {
    Played {
        winner: ParticipantId,
        score: SeriesScore,
    },
    /// W.O.: o adversário não compareceu ou desistiu.
    Walkover { winner: ParticipantId },
}

impl Outcome {
    pub fn winner(&self) -> ParticipantId {
        match *self {
            Outcome::Played { winner, .. } | Outcome::Walkover { winner } => winner,
        }
    }
}

/// Resultado enviado para uma partida.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchReport {
    pub match_id: MatchId,
    pub outcome: Outcome,
}

/// Frags somados de uma série jogada (os mapas que valem), do vencedor e do
/// perdedor da série. Quem venceu a série pode ter feito menos frags.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeriesFrags {
    pub winner: u32,
    pub loser: u32,
}

/// Resultado de uma série com os frags dela, como entra no histórico de uma
/// fase (`formats::StageHistory`). Os frags só pesam nos formatos de tabela
/// (último desempate antes do seed); a eliminação os ignora, e o W.O. também.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageReport {
    pub report: MatchReport,
    /// `None` quando algum mapa da série veio sem frags (são opcionais no
    /// envio): a série fica sem frags, em vez de contar 0.
    pub frags: Option<SeriesFrags>,
}

impl From<MatchReport> for StageReport {
    /// O resultado sem frags.
    fn from(report: MatchReport) -> Self {
        Self {
            report,
            frags: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MatchState {
    /// Algum lado ainda depende de outra partida.
    Pending,
    /// Os dois lados são conhecidos: a partida pode ser jogada.
    Ready { a: ParticipantId, b: ParticipantId },
    /// Pelo menos um lado é bye. `advanced` é quem avançou sem jogar; `None`
    /// quando os dois lados são bye (a partida também propaga um bye). Nos
    /// formatos de tabela, `advanced` é quem não joga na rodada: a folga do
    /// round robin (não vale ponto) ou o bye do suíço (vale vitória).
    Bye { advanced: Option<ParticipantId> },
    Completed {
        a: ParticipantId,
        b: ParticipantId,
        outcome: Outcome,
    },
    /// Partida condicional que não precisou acontecer (reset da grande final
    /// quando o campeão da chave de cima vence a primeira final).
    NotNeeded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Match {
    pub id: MatchId,
    pub a: Slot,
    pub b: Slot,
    /// Série melhor de N mapas (sempre ímpar).
    pub best_of: u8,
    pub state: MatchState,
}

/// Colocação final. Empatados dividem uma faixa: `rank` 5 com `tied` 4 é 5º–8º.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Placement {
    pub participant: ParticipantId,
    pub rank: u16,
    pub tied: u16,
}

/// Erros com variantes estáveis: a API converte cada uma num código para o
/// front traduzir. A mensagem (`Display`) é só para log.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum EngineError {
    #[error("precisa de pelo menos {min} participantes, recebeu {got}")]
    NotEnoughParticipants { min: usize, got: usize },
    /// Nos times: as vagas (em jogadores) não formam dois times; `min` é
    /// `2 × tamanho`.
    #[error("precisa de pelo menos {min} jogadores (dois times), recebeu {got}")]
    NotEnoughPlayers { min: usize, got: usize },
    #[error("aceita no máximo {max} participantes, recebeu {got}")]
    TooManyParticipants { max: usize, got: usize },
    #[error("participante repetido: {0:?}")]
    DuplicateParticipant(ParticipantId),
    #[error("melhor de {0} inválido: precisa ser ímpar e maior que zero")]
    InvalidBestOf(u8),
    #[error("partida inexistente: {0:?}")]
    UnknownMatch(MatchId),
    #[error("partida ainda não pode ser jogada: {0:?}")]
    MatchNotReady(MatchId),
    #[error("partida já tem resultado: {0:?}")]
    MatchAlreadyReported(MatchId),
    #[error("{participant:?} não joga a partida {match_id:?}")]
    NotInMatch {
        match_id: MatchId,
        participant: ParticipantId,
    },
    #[error("placar {score:?} inválido para melhor de {best_of}")]
    InvalidScore { best_of: u8, score: SeriesScore },
    #[error("o mapa {game} do detalhe está empatado")]
    GameTied { game: usize },
    #[error("mapa repetido na série: {0:?}")]
    MapRepeated(MapId),
    #[error("os frags do mapa {game} não batem com o vencedor")]
    FragsDisagree { game: usize },
    #[error("a série já está decidida pelos mapas enviados")]
    SeriesDecided,
    #[error("o próximo mapa da série é o {next}")]
    GameOutOfOrder { next: usize },
    #[error("a série melhor de {best_of} já teve todos os mapas")]
    NoGamesLeft { best_of: u8 },
    #[error("no pior caso são necessários {required} campeões; o pool tem {available}")]
    ChampionPoolExhausted { required: u32, available: u32 },
    #[error("o torneio precisa de pelo menos uma fase")]
    NoStages,
    #[error("número de rodadas {got} inválido: de 1 a {max}")]
    InvalidRounds { max: u16, got: u16 },
    /// O pareamento gravado não fecha com a fase: participante desconhecido
    /// ou repetido na rodada, rodada sem partida ou além do total, pareamento
    /// num formato que não pareia. Erro de consistência do histórico, não do
    /// usuário.
    #[error("pareamento gravado inválido na rodada {round}")]
    InvalidPairing { round: u16 },
}
