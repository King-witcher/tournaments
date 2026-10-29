//! O endereço de um webhook do Discord. A API só aceita
//! `https://discord.com/api/webhooks/{id}/{token}` (e as variantes
//! `discordapp.com`, `ptb.` e `canary.`): como a API faz requisições para o
//! endereço, qualquer outro host seria uma porta para pedir coisas a
//! serviços internos. Guarda-se o id e o token separados, nunca o endereço.

/// Os dois pedaços de um endereço de webhook.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebhookUrl {
    pub id: String,
    pub token: String,
}

const HOSTS: [&str; 6] = [
    "discord.com",
    "ptb.discord.com",
    "canary.discord.com",
    "discordapp.com",
    "ptb.discordapp.com",
    "canary.discordapp.com",
];

impl WebhookUrl {
    /// Lê o endereço colado pelo owner (com espaços em volta, se vierem).
    /// `None` para qualquer outra forma: outro host, `http`, porta, usuário,
    /// `?query` ou `#fragmento`, id que não é número, token fora do alfabeto.
    pub fn parse(text: &str) -> Option<Self> {
        let rest = text.trim().strip_prefix("https://")?;
        let (host, path) = rest.split_once('/')?;
        if !HOSTS.contains(&host) {
            return None;
        }
        let mut parts = path.strip_prefix("api/webhooks/")?.split('/');
        let (id, token) = (parts.next()?, parts.next()?);
        if parts.next().is_some() {
            return None;
        }
        let id_ok = (1..=20).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_digit());
        let token_ok = (20..=100).contains(&token.len())
            && token
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-');
        (id_ok && token_ok).then(|| Self {
            id: id.to_owned(),
            token: token.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN: &str = "abcDEF123_-abcDEF123_-abcDEF123_-abcDEF123_-abcDEF123_-abcDEF123_-abcd";

    fn url(host: &str) -> String {
        format!("https://{host}/api/webhooks/1234567890/{TOKEN}")
    }

    #[test]
    fn accepts_the_discord_hosts() {
        for host in HOSTS {
            let parsed = WebhookUrl::parse(&url(host)).unwrap_or_else(|| panic!("{host}"));
            assert_eq!(parsed.id, "1234567890");
            assert_eq!(parsed.token, TOKEN);
        }
        assert!(WebhookUrl::parse(&format!("  {}\n", url("discord.com"))).is_some());
    }

    #[test]
    fn refuses_everything_else() {
        let bad = [
            url("evil.com"),
            url("discord.com.evil.com"),
            url("discord.com:8443"),
            url("user@discord.com"),
            url("cdn.discord.com"),
            url("localhost"),
            url("127.0.0.1"),
            url("discord.com").replace("https", "http"),
            format!("{}?wait=true", url("discord.com")),
            format!("{}#x", url("discord.com")),
            format!("{}/slack", url("discord.com")),
            format!("{}/", url("discord.com")),
            url("discord.com").replace("1234567890", "12ab"),
            url("discord.com").replace("api/webhooks", "api/v10/webhooks"),
            "https://discord.com/api/webhooks/1/short".into(),
            format!("https://discord.com/api/webhooks//{TOKEN}"),
            String::new(),
        ];
        for text in bad {
            assert_eq!(WebhookUrl::parse(&text), None, "{text}");
        }
    }
}
