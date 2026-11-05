//! Os caminhos que denunciam um scanner e quem nunca é banido por eles. A mesma
//! lista está em `web/src/features/scanners/lib/traps.ts`: mudou aqui, mude lá.

use axum::http::{HeaderMap, header};

/// Extensões de arquivo que este site não serve: outras stacks, segredos e backups.
const EXTENSIONS: &[&str] = &[
    "php", "php3", "php4", "php5", "php7", "phtml", "phar", "asp", "aspx", "ashx", "jsp", "jspx",
    "action", "cgi", "cfm", "env", "sql", "sqlite", "db", "mdb", "bak", "old", "orig", "save",
    "swp", "ini", "conf", "cfg", "log", "pem", "key", "p12", "tar", "gz", "tgz", "rar", "7z",
    "zip", "dump",
];

/// Pedaços de caminho de produtos que este site não usa.
const NAMES: &[&str] = &[
    "wp-admin",
    "wp-content",
    "wp-includes",
    "wp-json",
    "wordpress",
    "phpmyadmin",
    "pma",
    "myadmin",
    "adminer",
    "cgi-bin",
    "boaform",
    "hnap1",
    "actuator",
    "server-status",
    "server-info",
    "vendor",
    "phpunit",
    "owa",
    "ecp",
    "autodiscover",
    "solr",
    "geoserver",
    "_ignition",
    "telescope",
    "jenkins",
    "nacos",
    "druid",
    "sslvpn",
    "dana-na",
    "global-protect",
];

/// Crawlers e leitores de link que dizem quem são. Um link para `/wp-admin` em
/// qualquer site não pode banir o Google (o site sairia da busca) nem o Discord
/// (os links do site perderiam a prévia). Quem forja o nome só escapa do ban.
const CRAWLERS: &[&str] = &[
    "google",
    "bing",
    "applebot",
    "duckduckbot",
    "yandexbot",
    "baiduspider",
    "discordbot",
    "twitterbot",
    "facebookexternalhit",
    "whatsapp",
    "telegrambot",
    "slackbot",
    "linkedinbot",
];

/// O caminho é um dos que só scanner pede: algum pedaço começa com ponto (`.env`,
/// `.git`, `..`), tem uma das extensões ou é um dos nomes. `/.well-known` fica de
/// fora: é onde ficam `security.txt`, a validação de certificado e afins.
pub fn is_trap(path: &str) -> bool {
    let path = path.to_ascii_lowercase().replace("%2e", ".");
    let mut segments = path.split('/').filter(|segment| !segment.is_empty());
    if path.starts_with("/.well-known/") || path == "/.well-known" {
        return false;
    }
    segments.any(|segment| {
        segment.starts_with('.')
            || NAMES.contains(&segment)
            || segment
                .rsplit_once('.')
                .is_some_and(|(_, extension)| EXTENSIONS.contains(&extension))
    })
}

/// Quem pediu pode ser banido. Navegador nunca: todos os atuais mandam
/// `Sec-Fetch-*` e os scanners não, e sem isto um link ou um `<img>` para
/// `/wp-admin` tiraria o site de um adversário no dia do torneio.
pub fn is_scanner(headers: &HeaderMap) -> bool {
    let browser = headers.contains_key("sec-fetch-site") || headers.contains_key("sec-fetch-mode");
    let agent = headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    !browser && !CRAWLERS.iter().any(|crawler| agent.contains(crawler))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_scanners_ask_for_is_a_trap() {
        for path in [
            "/.env",
            "/.env.local",
            "/.git/config",
            "/api/.env",
            "/.aws/credentials",
            "/wp-login.php",
            "/blog/xmlrpc.PHP",
            "/index.php/foo",
            "/wp-admin",
            "/wordpress/wp-admin/setup-config.php",
            "/cgi-bin/luci",
            "/vendor/phpunit/phpunit/src/Util/eval-stdin.php",
            "/backup.sql",
            "/site.tar.gz",
            "/config.env",
            "/%2eenv",
            "/%2e%2e/%2e%2e/etc/passwd",
            "/../../etc/passwd",
        ] {
            assert!(is_trap(path), "{path}");
        }
    }

    #[test]
    fn what_the_site_serves_is_not_a_trap() {
        for path in [
            "/",
            "/health",
            "/openapi.json",
            "/docs",
            "/tenants/ragecon/tournaments/copa",
            "/veto-rooms/4kZ9xQ",
            "/.well-known/security.txt",
            "/.well-known/acme-challenge/abc",
            "/robots.txt",
            "/sitemap.xml",
            "/favicon.ico",
            "/sw.js",
            "/manifest.webmanifest",
            "/images/logo.webp",
            // Só o nome inteiro conta.
            "/tenants/vendors",
            "/php",
        ] {
            assert!(!is_trap(path), "{path}");
        }
    }

    fn headers(pairs: &[(&'static str, &'static str)]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (name, value) in pairs {
            headers.insert(*name, value.parse().unwrap());
        }
        headers
    }

    #[test]
    fn browsers_and_named_crawlers_are_not_scanners() {
        assert!(!is_scanner(&headers(&[("sec-fetch-site", "cross-site")])));
        assert!(!is_scanner(&headers(&[("sec-fetch-mode", "no-cors")])));
        for agent in [
            "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)",
            "Mozilla/5.0 (compatible; Discordbot/2.0; +https://discordapp.com)",
            "WhatsApp/2.23",
        ] {
            assert!(!is_scanner(&headers(&[("user-agent", agent)])), "{agent}");
        }
    }

    #[test]
    fn anything_else_is_a_scanner() {
        assert!(is_scanner(&headers(&[])));
        for agent in [
            "curl/8.5.0",
            "python-requests/2.31",
            "Mozilla/5.0 zgrab/0.x",
        ] {
            assert!(is_scanner(&headers(&[("user-agent", agent)])), "{agent}");
        }
    }
}
