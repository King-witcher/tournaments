use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{shared::Locale, tenants::MyTenant, tournaments::TournamentStatus};

/// O link de convite de uma comunidade ou o de inscrição de um torneio.
#[derive(Serialize, ToSchema)]
pub struct InviteLink {
    /// Token do link `/invite/{token}`.
    #[schema(example = "7Fq2xKpR9mWbTnVc3hJdYe")]
    pub token: String,
}

/// O que quem recebeu o link vê antes de aceitar.
#[derive(Serialize, ToSchema)]
pub struct InvitePreview {
    /// Endereço da comunidade (para quem já é membro ir direto a ela).
    pub tenant_slug: String,
    pub tenant_name: String,
    /// A descrição da comunidade, em texto simples; nulo sem descrição.
    pub tenant_description: Option<String>,
    /// O logo da comunidade, no CDN; nulo sem logo.
    pub tenant_logo_url: Option<String>,
    /// O apelido do dono atual na comunidade (a comunidade não grava quem a
    /// criou).
    pub owner_name: String,
    /// Membros ativos da comunidade.
    pub member_count: i64,
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
    /// A descrição do organizador, em texto simples; vazia sem descrição.
    pub description: String,
    pub translations: Vec<InvitedTranslation>,
    pub status: TournamentStatus,
    pub max_participants: i32,
    pub participants: i64,
    /// Início previsto (sempre definido fora do rascunho).
    pub starts_at: Option<DateTime<Utc>>,
    /// A capa (2:1), no CDN; nulo sem capa.
    pub cover_url: Option<String>,
}

/// Nome e descrição do torneio do link numa tradução.
#[derive(Serialize, ToSchema)]
pub struct InvitedTranslation {
    pub locale: Locale,
    pub name: String,
    pub description: String,
}

#[derive(Serialize, ToSchema)]
pub struct AcceptedInvite {
    /// O tenant em que a pessoa está (quem já era membro mantém papel e apelido).
    pub tenant: MyTenant,
    /// Slug do torneio em que ficou inscrita, no link de inscrição.
    pub tournament: Option<String>,
}
