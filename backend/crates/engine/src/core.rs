//! Tipos compartilhados pelos formatos.

use serde::{Deserialize, Serialize};

/// Quem ocupa um lado da chave: um time, em todos os modos (no duelo, um time
/// de um jogador). É o id interno do time (`teams.id` na API, bigint).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ParticipantId(pub i64);

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
}

/// Identificador estável e legível de uma partida ("chave de cima, rodada 2,
/// partida 3"). Rodada e índice começam em 1.
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MatchState {
    /// Algum lado ainda depende de outra partida.
    Pending,
    /// Os dois lados são conhecidos: a partida pode ser jogada.
    Ready { a: ParticipantId, b: ParticipantId },
    /// Pelo menos um lado é bye. `advanced` é quem avançou sem jogar; `None`
    /// quando os dois lados são bye (a partida também propaga um bye).
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
}
