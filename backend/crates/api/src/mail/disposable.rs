//! Domínios recusados: descartáveis (a lista aberta em
//! `disposable_domains.txt`, atualizada por `scripts/disposable-domains.ts`) e
//! serviços de apelido, que dão endereços sem fim para uma caixa só e não
//! entram naquela lista porque não são descartáveis.

use std::{collections::HashSet, sync::LazyLock};

/// Serviços de apelido (relay): cada endereço novo abriria outra conta.
const RELAYS: &[&str] = &[
    "8alias.com",
    "8shield.net",
    "33mail.com",
    "aleeas.com",
    "anonaddy.com",
    "anonaddy.me",
    "dralias.com",
    "duck.com",
    "mozmail.com",
    "passinbox.com",
    "passmail.com",
    "passmail.net",
    "privaterelay.appleid.com",
    "silomails.com",
    "simplelogin.co",
    "simplelogin.com",
    "slmail.me",
    "slmails.com",
];

static BLOCKED: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    include_str!("disposable_domains.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .chain(RELAYS.iter().copied())
        .collect()
});

/// O domínio (em minúsculas) ou um domínio acima dele está bloqueado:
/// `x.mailinator.com` cai junto com `mailinator.com`.
pub fn is_blocked(domain: &str) -> bool {
    let mut current = domain;
    loop {
        if BLOCKED.contains(current) {
            return true;
        }
        match current.split_once('.') {
            // Para antes do domínio de topo sozinho (`com`).
            Some((_, parent)) if parent.contains('.') => current = parent,
            _ => return false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_listed_domains_and_their_subdomains() {
        assert!(is_blocked("mailinator.com"));
        assert!(is_blocked("yopmail.com"));
        assert!(is_blocked("abc.mailinator.com"));
        assert!(is_blocked("duck.com"));
        assert!(is_blocked("fulano.anonaddy.com"));
    }

    #[test]
    fn lets_common_providers_through() {
        for domain in [
            "gmail.com",
            "outlook.com",
            "hotmail.com",
            "yahoo.com.br",
            "icloud.com",
            "proton.me",
            "uol.com.br",
            "bol.com.br",
            "terra.com.br",
        ] {
            assert!(!is_blocked(domain), "{domain}");
        }
    }
}
