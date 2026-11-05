// Os caminhos que denunciam um scanner e quem nunca é banido por eles. A mesma
// lista está em `backend/crates/api/src/scanners/traps.rs`: mudou aqui, mude lá.
// O motivo de cada regra está lá.

import { routing } from "@/i18n/routing";

const EXTENSIONS = new Set(
  "php php3 php4 php5 php7 phtml phar asp aspx ashx jsp jspx action cgi cfm env sql sqlite db mdb bak old orig save swp ini conf cfg log pem key p12 tar gz tgz rar 7z zip dump".split(
    " ",
  ),
);

const NAMES = new Set(
  "wp-admin wp-content wp-includes wp-json wordpress phpmyadmin pma myadmin adminer cgi-bin boaform hnap1 actuator server-status server-info vendor phpunit owa ecp autodiscover solr geoserver _ignition telescope jenkins nacos druid sslvpn dana-na global-protect".split(
    " ",
  ),
);

const CRAWLERS = [
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

const LOCALES = new Set<string>(routing.locales);

/**
 * O caminho é um dos que só scanner pede. Com o idioma na frente, é uma página do
 * site (uma comunidade pode se chamar `solr`), e scanner nenhum sabe do prefixo.
 */
export function isTrap(pathname: string): boolean {
  const path = pathname.toLowerCase().replaceAll("%2e", ".");
  if (path === "/.well-known" || path.startsWith("/.well-known/")) {
    return false;
  }
  const segments = path.split("/").filter(Boolean);
  if (LOCALES.has(segments[0])) {
    return false;
  }
  return segments.some((segment) => {
    const dot = segment.lastIndexOf(".");
    return (
      segment.startsWith(".") ||
      NAMES.has(segment) ||
      (dot > 0 && EXTENSIONS.has(segment.slice(dot + 1)))
    );
  });
}

/** Quem pediu pode ser banido: não é navegador nem crawler que diz quem é. */
export function isScanner(headers: Headers): boolean {
  if (headers.has("sec-fetch-site") || headers.has("sec-fetch-mode")) {
    return false;
  }
  const agent = (headers.get("user-agent") ?? "").toLowerCase();
  return !CRAWLERS.some((crawler) => agent.includes(crawler));
}
