//! Os provedores externos de login (`user_identities.provider`) e os códigos
//! de erro de cada um.

/// Provedor externo de login, entrando pelo OAuth dele.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provider {
    Discord,
    Google,
}

impl Provider {
    /// O nome no banco, nas rotas e no cookie do state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Discord => "discord",
            Self::Google => "google",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "discord" => Some(Self::Discord),
            "google" => Some(Self::Google),
            _ => None,
        }
    }

    /// O avatar da conta vem do Discord: entrar e vincular trocam pelo dele, e
    /// desvincular o tira. O Google não mexe no avatar: a foto da conta Google
    /// é da pessoa, não do jogador, e não vem para o site.
    pub const fn sets_avatar(self) -> bool {
        matches!(self, Self::Discord)
    }

    pub const fn codes(self) -> &'static ProviderCodes {
        match self {
            Self::Discord => &DISCORD,
            Self::Google => &GOOGLE,
        }
    }
}

/// Os códigos de erro de um provedor, `auth.<provedor>_<caso>`.
pub struct ProviderCodes {
    /// 503: sem client id e secret, o login com ele fica desligado.
    pub not_configured: &'static str,
    /// A pessoa recusou a autorização no provedor.
    pub denied: &'static str,
    /// O provedor não respondeu, recusou o code ou mandou o que não se lê.
    pub failed: &'static str,
    /// 409: a conta do provedor já é login de outra conta daqui.
    pub taken: &'static str,
    /// 409: a conta já tem um login desse provedor.
    pub already_linked: &'static str,
    /// 409: a conta não tem login desse provedor para desligar.
    pub not_linked: &'static str,
}

const DISCORD: ProviderCodes = ProviderCodes {
    not_configured: "auth.discord_not_configured",
    denied: "auth.discord_denied",
    failed: "auth.discord_failed",
    taken: "auth.discord_taken",
    already_linked: "auth.discord_already_linked",
    not_linked: "auth.discord_not_linked",
};

const GOOGLE: ProviderCodes = ProviderCodes {
    not_configured: "auth.google_not_configured",
    denied: "auth.google_denied",
    failed: "auth.google_failed",
    taken: "auth.google_taken",
    already_linked: "auth.google_already_linked",
    not_linked: "auth.google_not_linked",
};
