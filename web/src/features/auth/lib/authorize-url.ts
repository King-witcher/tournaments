import type { Provider } from "./providers";

/**
 * O client id público de cada provedor, para entrar sem passar pela API. Não
 * é segredo (vai na URL de autorização), mas precisa ser o mesmo do
 * `DISCORD_CLIENT_ID`/`GOOGLE_CLIENT_ID` da API, que troca o code. Entra no
 * código no build: cada `process.env.NEXT_PUBLIC_…` escrito por extenso.
 */
const CLIENT_IDS: Record<Provider, string | undefined> = {
  discord: process.env.NEXT_PUBLIC_DISCORD_CLIENT_ID || undefined,
  google: process.env.NEXT_PUBLIC_GOOGLE_CLIENT_ID || undefined,
};

/**
 * A URL de autorização para entrar com o provedor, com o state e o challenge
 * da aba, igual à que a API monta para vincular (`authorize_url` em
 * `auth/discord.rs` e `auth/google.rs`): mudou lá, mude aqui. O provedor volta
 * para a página de callback desta origem, a mesma que a API usa na troca
 * (`{WEB_ORIGIN}/auth/{provedor}/callback`). `undefined` sem o client id: o
 * login com o provedor está desligado.
 */
export function authorizeUrl(
  provider: Provider,
  state: string,
  codeChallenge: string,
): string | undefined {
  const clientId = CLIENT_IDS[provider];
  if (!clientId) return undefined;
  const redirectUri = `${window.location.origin}/auth/${provider}/callback`;
  const common = {
    response_type: "code",
    client_id: clientId,
    state,
    code_challenge: codeChallenge,
    code_challenge_method: "S256",
    redirect_uri: redirectUri,
  };
  if (provider === "discord") {
    const query = new URLSearchParams({ ...common, scope: "identify" });
    return `https://discord.com/oauth2/authorize?${query}`;
  }
  const query = new URLSearchParams({
    ...common,
    scope: "openid profile",
    prompt: "select_account",
  });
  return `https://accounts.google.com/o/oauth2/v2/auth?${query}`;
}
