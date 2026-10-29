//! Uma partida vista de fora da chave, para o domínio `screenshots`: quem joga,
//! os mapas enviados (número e se já confirmou) e o próximo, que ainda não foi
//! enviado mas já recebe screenshot (o jogador anexa antes de enviar). As
//! confirmações por tempo vencidas valem antes, como em toda leitura.

use engine::core::{Match, MatchState};
use sqlx::PgConnection;

use super::{detail, due, games::GameStatus, key, stage, tasks};
use crate::{
    shared::ApiError, teams::StoredTeam, tenants::TenantMember, tournaments::StoredTournament,
};

pub struct MatchGames {
    pub tournament: StoredTournament,
    /// Posição da fase.
    pub stage: i16,
    /// `wb-1-3`.
    pub key: String,
    /// `user_id` dos jogadores dos dois times, quando já se sabe quem joga
    /// (partida pronta ou decidida); vazio antes.
    pub players: Vec<i64>,
    /// Os mapas enviados, na ordem.
    pub games: Vec<SlotGame>,
    /// O próximo mapa que dá para enviar; `None` com a série decidida ou o
    /// torneio fora de andamento.
    pub next_game: Option<i16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotGame {
    pub number: i16,
    pub confirmed: bool,
}

impl MatchGames {
    /// O mapa `number` já enviado.
    pub fn game(&self, number: i16) -> Option<SlotGame> {
        self.games.iter().copied().find(|g| g.number == number)
    }

    /// `user_id` joga a partida (é de um dos times).
    pub fn plays(&self, user_id: i64) -> bool {
        self.players.contains(&user_id)
    }
}

/// A partida `key` do torneio `slug` como `member` a vê (sem travar).
pub async fn games<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    slug: &str,
    key: &str,
) -> Result<MatchGames, ApiError> {
    let tournament = due::find_current(conn, member, slug).await?;
    load(conn, tournament, key).await
}

/// Como `games`, travando o torneio até o fim da transação (para quem grava).
pub async fn lock_games<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    slug: &str,
    key: &str,
) -> Result<MatchGames, ApiError> {
    let tournament = due::lock_current(conn, member, slug).await?;
    load(conn, tournament, key).await
}

async fn load(
    conn: &mut PgConnection,
    tournament: StoredTournament,
    key: &str,
) -> Result<MatchGames, ApiError> {
    let stage = stage::load(conn, &tournament).await?;
    let m: &Match = detail::find(&stage, key)?;
    let players = match m.state {
        MatchState::Ready { a, b } | MatchState::Completed { a, b, .. } => [a, b]
            .iter()
            .filter_map(|p| stage.entrant(*p))
            .flat_map(StoredTeam::user_ids)
            .collect(),
        _ => Vec::new(),
    };
    let games = stage.games_of(m.id);
    Ok(MatchGames {
        stage: stage.number,
        key: key::format(m.id),
        players,
        next_game: tasks::next_game(tournament.status, m, &games),
        games: games
            .iter()
            .map(|g| SlotGame {
                number: g.number,
                confirmed: g.status == GameStatus::Confirmed,
            })
            .collect(),
        tournament,
    })
}
