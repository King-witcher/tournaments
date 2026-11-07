//! A entrada de um inscrito num torneio (`MyTournament`): a situação, a
//! colocação, a próxima partida (só de quem vê) e o time. A mesma para a
//! lista de quem vê (`/me/tournaments`), para um torneio só (`.../mine`) e
//! para o perfil de um membro (`members`), com as idas ao banco juntas para
//! todos os torneios: o time dele (TDM) numa query (`teams::of_user_in`),
//! onde terminou nos encerrados em duas (`matches::final_places`); só os em
//! andamento leem a fase inteira, um a um (`matches::place`, que aplica as
//! confirmações e os passos de veto vencidos). A situação antes da chave sai
//! do status e do check-in, que a lista já trouxe.

use sqlx::PgConnection;

use super::types::{MyTournament, Situation};
use crate::{
    matches::{self, Place},
    shared::{ApiError, public_url},
    teams::{self, StoredTeam, UserTeam},
    tenants::TenantMember,
    tournaments::{
        StoredTournament, TournamentId, TournamentMode, TournamentStatus, TournamentSummary,
    },
};

/// Um torneio em que o inscrito está, ainda sem a situação.
pub struct Registered<'a, R> {
    /// Quem vê, no tenant do torneio.
    pub member: &'a TenantMember<R>,
    pub id: TournamentId,
    pub tournament: TournamentSummary,
    /// O check-in do inscrito (o `checked_in` de `tournament` é de quem vê).
    pub checked_in: bool,
    /// O torneio já lido por inteiro (a rota de um torneio só, para quem
    /// vê): a fase sai dele, sem lê-lo de novo.
    pub found: Option<StoredTournament>,
}

impl<R> Registered<'_, R> {
    fn teams(&self) -> bool {
        self.tournament.mode == TournamentMode::Tdm
    }

    /// Encerrado, da lista: a colocação vem de `final_places`, junto com a
    /// dos outros.
    fn settled(&self) -> bool {
        self.found.is_none() && self.tournament.status == TournamentStatus::Finished
    }
}

/// As entradas de `user_id` (quem vê, ou outro membro no perfil dele) em
/// cada torneio de `list`, na mesma ordem.
pub async fn entries<R>(
    conn: &mut PgConnection,
    user_id: i64,
    list: Vec<Registered<'_, R>>,
) -> Result<Vec<MyTournament>, ApiError> {
    let tdm: Vec<TournamentId> = list.iter().filter(|r| r.teams()).map(|r| r.id).collect();
    let settled: Vec<TournamentId> = list.iter().filter(|r| r.settled()).map(|r| r.id).collect();
    let mut user_teams = if tdm.is_empty() {
        Vec::new()
    } else {
        teams::of_user_in(conn, &tdm, user_id).await?
    }
    .into_iter();
    let mut finals = if settled.is_empty() {
        Vec::new()
    } else {
        matches::final_places(conn, &settled, user_id).await?
    }
    .into_iter();

    let mut out = Vec::with_capacity(list.len());
    for r in list {
        let user_team = if r.teams() { user_teams.next() } else { None };
        let final_place = if r.settled() {
            finals.next().flatten()
        } else {
            None
        };
        out.push(entry(conn, user_id, r, user_team, final_place).await?);
    }
    Ok(out)
}

/// A entrada de `user_id` em `r`, com o time dele (TDM) e, no encerrado da
/// lista, o lugar em que terminou e a fase (`None`: lê a fase).
async fn entry<R>(
    conn: &mut PgConnection,
    user_id: i64,
    r: Registered<'_, R>,
    user_team: Option<UserTeam>,
    final_place: Option<(Place, i16)>,
) -> Result<MyTournament, ApiError> {
    let teams = r.teams();
    let Registered {
        member,
        mut tournament,
        checked_in,
        found,
        ..
    } = r;
    let team = user_team
        .as_ref()
        .and_then(|u| u.team.as_ref())
        .map(StoredTeam::view);
    let formed = user_team.is_some_and(|u| u.any_formed);
    let before = Situation::before_bracket(tournament.status, checked_in);
    let stage_of = |n: i16| u16::try_from(n).ok();
    let (situation, stage, placement, next_match) = match (before, final_place) {
        (Some(situation), _) => (
            situation.or_reserve(teams && team.is_none() && formed),
            None,
            None,
            None,
        ),
        (None, Some((place, stage))) => {
            let (situation, placement) = Situation::in_bracket(place, teams);
            (situation, stage_of(stage), placement, None)
        }
        (None, None) => {
            // A próxima partida é só de quem vê.
            let place = if user_id != member.user.id {
                matches::place_of(conn, member, &tournament.slug, user_id).await?
            } else if let Some(found) = found {
                matches::place_from(conn, member, found).await?
            } else {
                matches::place(conn, member, &tournament.slug).await?
            };
            tournament.status = place.tournament.status;
            let (situation, placement) = Situation::in_bracket(place.place, teams);
            (situation, stage_of(place.stage), placement, place.next)
        }
    };
    Ok(MyTournament {
        tenant_slug: member.slug.clone(),
        tenant_name: member.name.clone(),
        tenant_logo_url: member.logo_key.as_deref().and_then(public_url),
        tournament,
        situation,
        stage,
        placement,
        next_match,
        team,
    })
}
