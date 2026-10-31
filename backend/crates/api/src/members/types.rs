use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    my_tournaments::{MyPlacement, Situation},
    teams::Team,
    tenants::Role,
    tournaments::TournamentSummary,
};

#[derive(Serialize, ToSchema)]
pub struct Member {
    pub user_id: i64,
    pub nickname: String,
    /// Nome exibido do usuário: o do Discord ou o do cadastro.
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub role: Role,
    pub joined_at: DateTime<Utc>,
    /// Por qual link entrou. Só para quem vê como organizer ou owner (para
    /// player é sempre `null`); `null` também para quem não entrou por
    /// convite (o owner) ou se o convite sumiu.
    pub joined_via: Option<JoinedVia>,
}

/// O link de convite pelo qual um membro entrou.
#[derive(Serialize, ToSchema)]
pub struct JoinedVia {
    /// Os 4 primeiros caracteres do token do link; nulo nos links criados
    /// antes de existir o `hint`.
    pub hint: Option<String>,
    /// Nome (no idioma original) do torneio, quando o link era de inscrição;
    /// nulo no convite da comunidade.
    pub tournament: Option<String>,
}

/// Um membro na comunidade, com os torneios dela em que se inscreveu.
#[derive(Serialize, ToSchema)]
pub struct MemberProfile {
    pub user_id: i64,
    pub nickname: String,
    /// Nome exibido do usuário: o do Discord ou o do cadastro.
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub role: Role,
    pub joined_at: DateTime<Utc>,
    /// Os torneios da comunidade em que ele se inscreveu, só os que quem vê
    /// vê na lista dela, do início mais recente ao mais antigo (os sem data,
    /// só rascunhos, primeiro; no empate, o criado por último).
    pub tournaments: Vec<MemberTournament>,
}

/// Um torneio em que o membro se inscreveu.
#[derive(Serialize, ToSchema)]
pub struct MemberTournament {
    /// Como na lista da comunidade: `registered` e `checked_in` são de quem
    /// vê, não do membro.
    pub tournament: TournamentSummary,
    /// A situação do membro no torneio.
    pub situation: Situation,
    /// Colocação do membro, se foi eliminado, desistiu ou venceu, quando já
    /// se definiu.
    pub placement: Option<MyPlacement>,
    /// TDM: o time do membro (`null` no duelo e sem time).
    pub team: Option<Team>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateMyMembership {
    /// 1 a 32 caracteres; espaços nas pontas são removidos.
    pub nickname: String,
}

/// Papéis que o owner pode dar. A posse do tenant não muda por aqui.
#[derive(Deserialize, ToSchema, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum AssignableRole {
    Organizer,
    Player,
}

impl From<AssignableRole> for Role {
    fn from(role: AssignableRole) -> Self {
        match role {
            AssignableRole::Organizer => Self::Organizer,
            AssignableRole::Player => Self::Player,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct SetRole {
    pub role: AssignableRole,
}

/// `{user_id}` de `/tenants/{slug}/members/{user_id}/...` (o slug fica com `TenantMember`).
#[derive(Deserialize)]
pub struct MemberPath {
    pub user_id: i64,
}
