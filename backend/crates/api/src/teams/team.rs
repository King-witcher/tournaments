//! Um time como a API o usa: quem ocupa um lado da chave (o `ParticipantId`
//! do `engine`), com os jogadores. No duelo, o time de um jogador.

use chrono::{DateTime, Utc};
use engine::core::ParticipantId;

use super::types::{Team, TeamMember};

/// Um time do torneio, com os inscritos dele (o capitão primeiro, depois na
/// ordem de inscrição).
#[derive(Debug, Clone)]
pub struct StoredTeam {
    /// `teams.id`: o `ParticipantId` do `engine`.
    pub id: i64,
    /// `None` no duelo e nos times sem nome.
    pub name: Option<String>,
    pub seed: Option<i32>,
    /// O rating do time congelado no início, em centésimos, para o menor
    /// rating (`lifecycle`); `None` antes do início e sem esse desempate.
    pub rating_at_start: Option<i32>,
    /// `user_id` do capitão (um dos `members`).
    pub captain: i64,
    /// Desistiu do torneio em andamento.
    pub withdrawn: bool,
    /// Quando desistiu.
    pub withdrawn_at: Option<DateTime<Utc>>,
    pub members: Vec<StoredMember>,
    /// Fases 2 em diante: a presença confirmada e o seed do time em cada uma
    /// (`stage_teams`), na ordem das fases.
    pub stages: Vec<StageTeam>,
}

/// O time numa fase da 2 em diante.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub struct StageTeam {
    pub stage: i16,
    /// Quando confirmou presença no check-in da fase.
    pub checked_in_at: Option<DateTime<Utc>>,
    /// O seed na fase: entrou nela.
    pub seed: Option<i32>,
}

/// Um jogador do time.
#[derive(Debug, Clone)]
pub struct StoredMember {
    pub user_id: i64,
    /// Apelido no tenant (vazio: a conta foi excluída).
    pub nickname: String,
    pub avatar_url: Option<String>,
    /// Saiu da comunidade (ou foi removido).
    pub left: bool,
}

/// Os times da fase `stage` de `teams` (os da fase 1, de `seeded`), na ordem
/// dos seeds dela e com o seed dela em `seed`: dentro da fase, é o que vale
/// (lado `a`, quem começa o veto, a ordem da chave no desempate).
pub fn stage_entrants(teams: &[StoredTeam], stage: i16) -> Vec<StoredTeam> {
    if stage == 1 {
        return teams.to_vec();
    }
    let mut entrants: Vec<StoredTeam> = teams
        .iter()
        .filter_map(|t| {
            let seed = t.stage_seed(stage)?;
            Some(StoredTeam {
                seed: Some(seed),
                ..t.clone()
            })
        })
        .collect();
    entrants.sort_by_key(|t| t.seed);
    entrants
}

impl StoredTeam {
    pub const fn participant(&self) -> ParticipantId {
        ParticipantId(self.id)
    }

    /// A linha do time na fase `stage` (2 em diante), se há.
    pub fn in_stage(&self, stage: i16) -> Option<&StageTeam> {
        self.stages.iter().find(|s| s.stage == stage)
    }

    /// O seed do time na fase `stage`: o de `teams` na 1, o de `stage_teams`
    /// nas outras. `None`: não entrou nela.
    pub fn stage_seed(&self, stage: i16) -> Option<i32> {
        if stage == 1 {
            self.seed
        } else {
            self.in_stage(stage).and_then(|s| s.seed)
        }
    }

    /// Confirmou presença no check-in da fase `stage`.
    pub fn checked_in_for(&self, stage: i16) -> bool {
        self.in_stage(stage)
            .is_some_and(|s| s.checked_in_at.is_some())
    }

    /// Desistiu antes de `at` (o início de uma fase); sem `at`, desistiu.
    pub fn withdrawn_before(&self, at: Option<DateTime<Utc>>) -> bool {
        match (self.withdrawn_at, at) {
            (Some(w), Some(at)) => w < at,
            (Some(_), None) => true,
            (None, _) => false,
        }
    }

    /// `user_id` joga por este time.
    pub fn has(&self, user_id: i64) -> bool {
        self.members.iter().any(|m| m.user_id == user_id)
    }

    /// Os `user_id` dos jogadores, para os avisos.
    pub fn user_ids(&self) -> Vec<i64> {
        self.members.iter().map(|m| m.user_id).collect()
    }

    /// Fora do torneio: desistiu ou nenhum jogador dele continua na
    /// comunidade. Toda partida pronta com ele vira W.O. para o adversário.
    pub fn out(&self) -> bool {
        self.withdrawn || self.members.iter().all(|m| m.left)
    }

    /// O nome que vai como texto nos avisos (`opponent`, pódio, Discord): o
    /// do time; sem nome, o apelido do capitão (no duelo, o único jogador). A
    /// tela monta o mesmo pelo `Team` ("Time de {capitão}" nos times sem
    /// nome).
    pub fn display_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| {
            self.members
                .iter()
                .find(|m| m.user_id == self.captain)
                .or(self.members.first())
                .map(|m| m.nickname.clone())
                .unwrap_or_default()
        })
    }

    /// Como a API mostra o time.
    pub fn view(&self) -> Team {
        Team {
            id: self.id,
            name: self.name.clone(),
            seed: self.seed,
            captain: self.captain,
            withdrawn: self.withdrawn,
            members: self
                .members
                .iter()
                .map(|m| TeamMember {
                    user_id: m.user_id,
                    nickname: m.nickname.clone(),
                    avatar_url: m.avatar_url.clone(),
                    left: m.left,
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn member(user_id: i64, nickname: &str, left: bool) -> StoredMember {
        StoredMember {
            user_id,
            nickname: nickname.into(),
            avatar_url: None,
            left,
        }
    }

    fn team(name: Option<&str>, members: Vec<StoredMember>) -> StoredTeam {
        StoredTeam {
            id: 1,
            name: name.map(Into::into),
            seed: Some(1),
            rating_at_start: None,
            captain: members[0].user_id,
            withdrawn: false,
            withdrawn_at: None,
            members,
            stages: Vec::new(),
        }
    }

    #[test]
    fn out_when_withdrawn_or_everyone_left() {
        let duel = team(None, vec![member(7, "Ana", false)]);
        assert!(!duel.out());
        assert!(team(None, vec![member(7, "Ana", true)]).out());
        assert!(
            StoredTeam {
                withdrawn: true,
                ..duel
            }
            .out()
        );
        let two = team(None, vec![member(7, "Ana", true), member(8, "Bia", false)]);
        assert!(!two.out());
    }

    #[test]
    fn display_name_is_the_name_or_the_captain() {
        assert_eq!(
            team(None, vec![member(7, "Ana", false)]).display_name(),
            "Ana"
        );
        let named = team(Some("Os Bons"), vec![member(7, "Ana", false)]);
        assert_eq!(named.display_name(), "Os Bons");
        let two = team(None, vec![member(7, "Ana", false), member(8, "Bia", false)]);
        assert_eq!(two.display_name(), "Ana");
    }
}
