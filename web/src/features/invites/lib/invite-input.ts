import { routing } from "@/i18n/routing";

/** O código curto: 8 caracteres, minúsculas e dígitos sem os ambíguos. */
const SHORT_CODE = /^[2-9a-hjkmnp-z]{8}$/;
/** O token antigo: base58 de 22 caracteres, em que a caixa importa. */
const LONG_TOKEN = /^[1-9A-HJ-NP-Za-km-z]{22}$/;
const SITE_DOMAIN = "telefrag.com.br";

export type InviteInput =
  | { kind: "invite"; token: string }
  /** Outro endereço do site, sem o idioma: a navegação põe o de quem usa. */
  | { kind: "path"; path: string }
  | { kind: "invalid" };

/** O convite tem o formato curto (e dá para digitar). */
export function isShortCode(token: string) {
  return SHORT_CODE.test(token);
}

/**
 * Interpreta o que a pessoa digitou ou colou em "Link ou código": um código, um
 * link de convite ou outro endereço do próprio site. `currentHost` é o
 * `location.host` da página. Nunca devolve endereço de fora: o `path` é montado
 * só com os segmentos do caminho.
 */
export function parseInviteInput(
  input: string,
  currentHost: string,
): InviteInput {
  const text = input.trim();
  const token = inviteToken(text);
  if (token) return { kind: "invite", token };

  const url = siteUrl(text, currentHost);
  if (!url) return { kind: "invalid" };
  const segments = url.pathname.split("/").filter(Boolean);
  if ((routing.locales as readonly string[]).includes(segments[0])) {
    segments.shift();
  }
  if (segments[0] !== "invite") {
    const path = `/${segments.join("/")}${url.search}${url.hash}`;
    return { kind: "path", path };
  }
  const found = segments.length === 2 ? inviteToken(segments[1]) : undefined;
  return found ? { kind: "invite", token: found } : { kind: "invalid" };
}

/** O token, com a caixa em que está guardado; o código curto aceita maiúsculas. */
function inviteToken(text: string) {
  if (LONG_TOKEN.test(text)) return text;
  const code = text.toLowerCase();
  return SHORT_CODE.test(code) ? code : undefined;
}

/** `text` como URL `http(s)` deste site (a página atual ou o domínio dele). */
function siteUrl(text: string, currentHost: string) {
  if (!/^https?:\/\//i.test(text)) return undefined;
  try {
    const url = new URL(text);
    const own =
      url.host === currentHost ||
      url.hostname === SITE_DOMAIN ||
      url.hostname.endsWith(`.${SITE_DOMAIN}`);
    return own ? url : undefined;
  } catch {
    return undefined;
  }
}
