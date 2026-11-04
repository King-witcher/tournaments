//! Os tipos do veto: regras, eventos, passos, a vez, o contexto das opções
//! e os erros (ver o `//!` do módulo).

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::Action;
use crate::champions::{ChampionRules, NoRepeatWho};
use crate::core::{ChampionId, MapId, PlayerId};
use crate::series::Side;

/// Mapas oferecidos em cada votação (como no próprio Quake Champions).
pub const VOTE_OPTIONS: usize = 3;

/// Como sai o mapa de cada partida.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MapMode {
    Veto,
    Vote,
    Random,
    Off,
}

/// A ordem dos passos do veto de mapas (modo `Veto`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VetoOrder {
    /// Os lados banem, alternando, até sobrarem só os mapas da série; depois
    /// escolhem a ordem dos que sobraram, e o último é o decisivo.
    BansFirst,
    /// Dois bans, depois os picks e por fim o resto dos bans.
    Alternating,
}

/// A ordem das escolhas de campeão, depois dos bans (ver `team.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PickOrder {
    /// Um por vez, partida a partida, alternando os lados: O, X, X, O, O, X,
    /// X, O..., com quem abre (O) trocando a cada partida. No duelo, um passo
    /// por lado (`Event::Step`); nos times, um por jogador (`Event::TeamPick`).
    Alternating,
    /// A janela simultânea em qualquer tamanho, inclusive no duelo, com as
    /// escolhas de um lado escondidas dos outros até ela fechar. Só com
    /// espelho (`check`).
    Blind,
    /// A janela simultânea aberta, nos times: fica só para refazer as séries
    /// que começaram antes de a ordem existir.
    Open,
}

impl PickOrder {
    /// As escolhas são uma janela simultânea, sem vez.
    pub const fn window(self) -> bool {
        !matches!(self, Self::Alternating)
    }

    /// Às cegas exige espelho: sem ele, os dois lados poderiam pegar o mesmo
    /// campeão, e não há regra para isso. Erro: `BlindNeedsMirror`.
    pub const fn check(self, mirror: bool) -> Result<(), VetoError> {
        if matches!(self, Self::Blind) && !mirror {
            Err(VetoError::BlindNeedsMirror)
        } else {
            Ok(())
        }
    }
}

/// O que o torneio define para o veto das séries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VetoRules {
    pub map_mode: MapMode,
    /// A ordem dos passos do veto de mapas; só vale no modo `Veto`.
    pub order: VetoOrder,
    /// Pool do torneio, na ordem configurada (fora do modo `Off`).
    pub pool: Vec<MapId>,
    /// Draft de campeões no site, antes da partida 1.
    pub champion_phase: bool,
    /// A ordem das escolhas de campeão.
    pub pick_order: PickOrder,
    /// Jogadores por lado: 1 no duelo; nos times, a escolha é por jogador, um
    /// campeão por partida.
    pub team_size: u8,
    /// As regras de campeões do torneio: bans da série, espelho, repetir no
    /// time e não repetição (o escopo dela fica com a API, que monta
    /// `Context::used`).
    pub champions: ChampionRules,
}

impl VetoRules {
    /// Há alguma fase antes das partidas (mapa ou campeões).
    pub fn has_phases(&self) -> bool {
        self.map_mode != MapMode::Off || self.champion_phase
    }

    /// São times (mais de um jogador por lado).
    pub const fn teams(&self) -> bool {
        self.team_size > 1
    }

    /// Cada jogador escolhe o próprio campeão (`Event::TeamPick`): nos times
    /// e em toda janela. Só o duelo alternado escolhe por lado (`Event::Step`).
    pub const fn per_player(&self) -> bool {
        self.teams() || self.pick_order.window()
    }

    /// Bans de campeão de cada lado na série.
    pub fn bans(&self) -> u8 {
        u8::try_from(self.champions.bans_per_series).unwrap_or(u8::MAX)
    }

    /// Quem não repete campeão entre partidas, se alguém (no duelo, `Player`
    /// e `Team` dão no mesmo).
    pub fn no_repeat(&self) -> Option<NoRepeatWho> {
        self.champions.no_repeat.map(|r| r.who)
    }

    /// O pool aguenta séries de até `longest` partidas: sem mapa repetido e,
    /// fora do modo `Off`, com pelo menos um mapa por partida. Erros:
    /// `PoolRepeated`, `PoolTooSmall`.
    pub fn validate(&self, longest: u8) -> Result<(), VetoError> {
        let mut seen = HashSet::new();
        for map in &self.pool {
            if !seen.insert(*map) {
                return Err(VetoError::PoolRepeated(*map));
            }
        }
        if self.map_mode != MapMode::Off && self.pool.len() < usize::from(longest) {
            return Err(VetoError::PoolTooSmall {
                pool: self.pool.len(),
                best_of: longest,
            });
        }
        Ok(())
    }
}

/// O que um passo escolhe.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Choice {
    Map(MapId),
    Champion(ChampionId),
}

/// Um evento do histórico do veto de uma série, na ordem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// O veto começa (os dois prontos, ou o organizador forçou): `first` é o
    /// seed melhor, que começa o veto de mapas (os bans de campeão começam
    /// pelo outro); `seed`, a dos sorteios da série. Abre a partida 1.
    Begin { seed: u64, first: Side },
    /// A partida `game` (2 em diante) começa: a anterior foi enviada.
    /// `previous`, o mapa em que a anterior foi enviada, quando há: é ele que
    /// sai do pool da votação e do sorteio das seguintes (o organizador pode
    /// ter lançado a anterior sem o veto, ou em outro mapa).
    Game { game: u8, previous: Option<MapId> },
    /// Um passo de um lado: mapa, ban de campeão ou, no duelo, pick (para a
    /// partida que a vez diz).
    Step {
        side: Side,
        action: Action,
        choice: Choice,
    },
    /// O pick por jogador (`VetoRules::per_player`): `player`, do lado `side`,
    /// fica com `champion` na partida `game`. `player` é o jogador-alvo, de
    /// quem é o campeão, que quem grava guarda à parte de quem agiu (`actor`).
    /// `gave_way`: as opções vieram com algo cedido (ao vivo, o engine calcula;
    /// no `replay`, vale o gravado).
    TeamPick {
        side: Side,
        player: PlayerId,
        game: u8,
        champion: ChampionId,
        gave_way: bool,
    },
    /// Desfaz o último passo da partida em andamento.
    Undo,
}

/// Um passo que vale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Applied {
    pub side: Side,
    pub action: Action,
    pub choice: Choice,
    /// No pick por jogador, o jogador.
    pub player: Option<PlayerId>,
    /// No pick de campeão, a partida a que ele se destina.
    pub game: Option<u8>,
    /// No pick por jogador: as opções vieram com algo cedido (a escolha pode
    /// repetir a de outro sem que uma das duas esteja errada).
    pub gave_way: bool,
}

impl Applied {
    /// O campeão do passo; `None` no de mapa.
    pub const fn champion(&self) -> Option<ChampionId> {
        match self.choice {
            Choice::Champion(c) => Some(c),
            Choice::Map(_) => None,
        }
    }
}

/// Sobre o que é o passo esperado.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    Map,
    Champion,
}

/// O que o veto espera agora.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Turn {
    /// Ainda não começou.
    NotBegun,
    /// A partida `game` espera `action` sobre `target` de `sides` (os dois
    /// na votação, enquanto nenhum votou; na janela simultânea, os lados em
    /// que falta alguém: `Veto::missing`). `pick_for`: no pick com vez, a
    /// partida a que ele se destina; `None` na janela, em que cada um escolhe
    /// a partida (`is_window`).
    Act {
        game: u8,
        action: Action,
        target: Target,
        sides: Vec<Side>,
        pick_for: Option<u8>,
    },
    /// O veto da partida `game` acabou: espera o envio dela.
    Done { game: u8 },
    /// A partida `game` chegou aos picks por jogador, mas o elenco de agora
    /// não foi informado (`Veto::set_rosters`): sem ele, não se sabe quem
    /// falta nem o que já foi enviado. O `replay` sozinho para aqui; a API
    /// sempre informa.
    RostersUnknown { game: u8 },
}

impl Turn {
    /// É a janela simultânea: sem vez, com o prazo único dela. O pick
    /// alternado por jogador é `TeamPick` com `pick_for`, e cada um é um passo.
    pub const fn is_window(&self) -> bool {
        matches!(
            self,
            Self::Act {
                action: Action::TeamPick,
                pick_for: None,
                ..
            }
        )
    }
}

/// O que as opções de campeão precisam saber de fora da série.
#[derive(Clone, Copy, Debug)]
pub struct Context<'a> {
    /// Os campeões do torneio, na ordem em que aparecem.
    pub champions: &'a [ChampionId],
    /// O que cada lado (`[a, b]`) já usou fora desta série, no escopo da
    /// regra de não repetição (vazio no escopo da série). Nos times, pela
    /// regra do time.
    pub used: [&'a [ChampionId]; 2],
    /// Nos times, pela regra do jogador: o que cada um já usou fora desta
    /// série, no escopo da regra.
    pub used_by: &'a [(PlayerId, ChampionId)],
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum VetoError {
    #[error("o pool tem {pool} mapas para uma série melhor de {best_of}")]
    PoolTooSmall { pool: usize, best_of: u8 },
    #[error("mapa repetido no pool: {0:?}")]
    PoolRepeated(MapId),
    #[error("o draft precisa de {required} campeões; o pool tem {available}")]
    ChampionPoolExhausted { required: usize, available: usize },
    #[error("escolha às cegas sem espelho")]
    BlindNeedsMirror,
    #[error("o veto já começou")]
    AlreadyBegun,
    #[error("o veto ainda não começou")]
    NotBegun,
    #[error("a próxima partida do veto é a {next}")]
    GameOutOfOrder { next: u8 },
    #[error("o draft de campeões da série ainda não acabou")]
    DraftPending,
    #[error("o veto desta partida já acabou")]
    Finished,
    #[error("não é a vez deste lado")]
    NotYourTurn,
    #[error("o passo esperado é outro")]
    WrongAction,
    #[error("escolha fora das opções")]
    OptionInvalid,
    #[error("campeão já usado, pela regra de não repetição")]
    ChampionForbidden,
    #[error("campeão já escolhido para esta partida por outro jogador")]
    ChampionTaken,
    #[error("nada para desfazer nesta partida")]
    NothingToUndo,
    #[error("este jogador já escolheu o campeão desta partida")]
    AlreadyPicked,
    #[error("o elenco de agora não foi informado")]
    RostersUnknown,
    #[error("jogador repetido no elenco: {0:?}")]
    RosterRepeated(PlayerId),
    #[error("elenco com {got} jogadores, o máximo é {max}")]
    RosterTooLarge { max: u8, got: usize },
}
