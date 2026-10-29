import { hasLocale, type Locale } from "next-intl";

import { routing } from "@/i18n/routing";
import { isProvider, type Provider } from "./providers";
import { safeReturnTo } from "./return-to";

const KEY = "oauth-flow";

/** O fluxo vale 10 minutos: depois disso, o code do provedor também já venceu. */
const TTL_MS = 10 * 60 * 1000;

/** Entrar com o provedor ou vinculá-lo à conta logada. */
export type OAuthIntent = "login" | "link";

/**
 * Um OAuth aberto nesta aba, com PKCE: o `state` volta do provedor no
 * callback, e o `verifier` vai à API para trocar o code (o provedor o confere
 * contra o challenge que foi na autorização). Só existe no `sessionStorage`
 * da aba, e o callback o apaga ao ler.
 */
export type OAuthFlow = {
  provider: Provider;
  intent: OAuthIntent;
  state: string;
  verifier: string;
  /** Caminho sem idioma para onde voltar. */
  returnTo: string;
  locale: Locale;
  at: number;
};

/** 32 bytes aleatórios em base64url sem `=`: 43 caracteres. */
function randomSecret() {
  return base64url(crypto.getRandomValues(new Uint8Array(32)));
}

function base64url(bytes: Uint8Array) {
  return btoa(String.fromCharCode(...bytes))
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/, "");
}

/** O challenge S256 do verifier: `BASE64URL(SHA-256(verifier))`. */
export async function codeChallenge(verifier: string) {
  const digest = await crypto.subtle.digest(
    "SHA-256",
    new TextEncoder().encode(verifier),
  );
  return base64url(new Uint8Array(digest));
}

/**
 * Abre um fluxo novo nesta aba (o anterior, se havia, deixa de valer). Sem
 * `sessionStorage` (bloqueado), lança: o login pelo provedor não tem como
 * voltar.
 */
export function startOAuthFlow(
  provider: Provider,
  intent: OAuthIntent,
  returnTo: string,
  locale: Locale,
): OAuthFlow {
  const flow: OAuthFlow = {
    provider,
    intent,
    state: randomSecret(),
    verifier: randomSecret(),
    returnTo,
    locale,
    at: Date.now(),
  };
  sessionStorage.setItem(KEY, JSON.stringify(flow));
  return flow;
}

/**
 * O fluxo aberto nesta aba, que sai do `sessionStorage` ao ler (vale uma
 * vez). `undefined` sem fluxo, vencido ou ilegível.
 */
export function takeOAuthFlow(): OAuthFlow | undefined {
  try {
    const raw = sessionStorage.getItem(KEY);
    sessionStorage.removeItem(KEY);
    if (!raw) return undefined;
    const flow = JSON.parse(raw) as Partial<OAuthFlow>;
    const fresh = typeof flow.at === "number" && Date.now() - flow.at < TTL_MS;
    if (
      !fresh ||
      typeof flow.provider !== "string" ||
      !isProvider(flow.provider) ||
      (flow.intent !== "login" && flow.intent !== "link") ||
      typeof flow.state !== "string" ||
      typeof flow.verifier !== "string" ||
      !hasLocale(routing.locales, flow.locale)
    ) {
      return undefined;
    }
    return {
      ...(flow as OAuthFlow),
      returnTo: safeReturnTo(flow.returnTo),
    };
  } catch {
    return undefined;
  }
}
