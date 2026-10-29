//! Um time como a API o usa: quem ocupa um lado da chave (o `ParticipantId`
//! do `engine`), com os jogadores. No duelo, o time de um jogador.

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
    /// `user_id` do capitão (um dos `members`).
    pub captain: i64,
    /// Desistiu do torneio em andamento.
    pub withdrawn: bool,
    pub members: Vec<StoredMember>,
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

impl StoredTeam {
    pub const fn participant(&self) -> ParticipantId {
        ParticipantId(self.id)
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
            captain: members[0].user_id,
            withdrawn: false,
            members,
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
