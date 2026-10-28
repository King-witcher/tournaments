use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    shared::Locale,
    tenants::MyTenant,
    tournaments::{NameTranslation, TournamentStatus},
};

#[derive(Serialize, ToSchema)]
pub struct CreatedInvite {
    pub id: i64,
    /// Token do link `/invite/{token}`. Só aparece nesta resposta: o banco guarda o hash.
    #[schema(example = "7Fq2xKpR9mWbTnVc3hJdYe")]
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

/// Convite ativo (nem revogado nem expirado).
#[derive(Serialize, ToSchema)]
pub struct Invite {
    pub id: i64,
    /// Apelido, neste tenant, de quem criou.
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    /// Quantas pessoas entraram por este link.
    pub uses: i64,
}

/// Link de inscrição recém-criado. Sem validade: vale enquanto a inscrição
/// do torneio estiver aberta.
#[derive(Serialize, ToSchema)]
pub struct CreatedTournamentInvite {
    pub id: i64,
    /// Token do link `/invite/{token}`. Só aparece nesta resposta: o banco guarda o hash.
    #[schema(example = "7Fq2xKpR9mWbTnVc3hJdYe")]
    pub token: String,
}

/// Link de inscrição não revogado de um torneio.
#[derive(Serialize, ToSchema)]
pub struct TournamentInvite {
    pub id: i64,
    /// Apelido, neste tenant, de quem criou.
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    /// Quantas pessoas se inscreveram por este link.
    pub uses: i64,
}

/// `{invite_id}` de `/tenants/{slug}/invites/{invite_id}` (o slug fica com `TenantMember`).
#[derive(Deserialize)]
pub struct InvitePath {
    pub invite_id: i64,
}

/// O que quem recebeu o link vê antes de aceitar.
#[derive(Serialize, ToSchema)]
pub struct InvitePreview {
    /// Endereço da comunidade (para quem já é membro ir direto a ela).
    pub tenant_slug: String,
    pub tenant_name: String,
    /// Nulo no link de inscrição, que vale enquanto a inscrição estiver aberta.
    pub expires_at: Option<DateTime<Utc>>,
    /// Link de inscrição: aceitar também inscreve neste torneio.
    pub tournament: Option<InvitedTournament>,
    /// Quem abre o link e está logado: se já é (ou foi) membro. `null` sem
    /// login.
    pub viewer: Option<InviteViewer>,
}

/// Quem abre o convite, em relação à comunidade e ao torneio do link.
#[derive(Serialize, ToSchema)]
pub struct InviteViewer {
    pub membership: InviteMembership,
    /// Apelido na comunidade (o de antes, para quem saiu); `null` para quem
    /// nunca foi membro.
    pub nickname: Option<String>,
    /// No link de inscrição: já está inscrito no torneio.
    pub registered: bool,
}

#[derive(Serialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InviteMembership {
    /// Ainda não é membro.
    None,
    /// Membro ativo: aceitar não muda papel nem apelido (no link de
    /// inscrição, só inscreve).
    Member,
    /// Saiu ou foi removido: aceitar o traz de volta como player, com o
    /// apelido e o histórico de antes.
    Former,
}

/// O torneio de um link de inscrição. O link deixa de valer quando a
/// inscrição não está aberta (`status`) ou lota.
#[derive(Serialize, ToSchema)]
pub struct InvitedTournament {
    pub slug: String,
    pub locale: Locale,
    pub name: String,
    pub translations: Vec<NameTranslation>,
    pub status: TournamentStatus,
    pub max_participants: i32,
    pub participants: i64,
    /// Início previsto (sempre definido fora do rascunho).
    pub starts_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, ToSchema)]
pub struct AcceptedInvite {
    /// O tenant em que a pessoa está (quem já era membro mantém papel e apelido).
    pub tenant: MyTenant,
    /// Slug do torneio em que ficou inscrita, no link de inscrição.
    pub tournament: Option<String>,
}
