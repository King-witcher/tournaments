import { useLocale } from "next-intl";
import { useEffect, useState } from "react";

import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { authorizeUrl } from "../lib/authorize-url";
import {
  codeChallenge,
  type OAuthIntent,
  startOAuthFlow,
} from "../lib/oauth-flow";
import type { Provider } from "../lib/providers";

/**
 * Abre o OAuth do provedor, com PKCE: `start` abre o fluxo nesta aba (o state
 * e o code verifier ficam só no `sessionStorage` dela) e vai para o provedor,
 * que volta para `OAuthCallback` e dali para `returnTo`, neste idioma. Para
 * entrar, a URL de autorização sai daqui mesmo (`authorizeUrl`, com o client
 * id público), sem esperar a API. Para vincular, a API confere a sessão (e a
 * senha atual, na conta com senha), guarda o challenge e devolve a URL; ela
 * recusando, `error` traz a mensagem e `onError` recebe o código, e a pessoa
 * não sai do site. Sem o client id, entrar mostra o `not_configured` do
 * provedor, como a API. `pending` continua enquanto o navegador sai para o
 * provedor.
 */
export function useOAuthStart(
  provider: Provider,
  action: OAuthIntent,
  returnTo: string,
) {
  const locale = useLocale();
  const errorMessage = useApiErrorMessage();
  const { run, pending, error } = useApiAction();
  const [loginError, setLoginError] = useState<string>();
  const [leaving, setLeaving] = useState(false);

  // Voltando do provedor pelo "Voltar", a página pode vir do cache do
  // navegador como estava: com o botão ainda pendente.
  useEffect(() => {
    const reset = (event: PageTransitionEvent) => {
      if (event.persisted) setLeaving(false);
    };
    window.addEventListener("pageshow", reset);
    return () => window.removeEventListener("pageshow", reset);
  }, []);

  const leave = (url: string) => {
    setLeaving(true);
    window.location.assign(url);
  };

  const signIn = async () => {
    setLeaving(true);
    try {
      const flow = startOAuthFlow(provider, "login", returnTo, locale);
      const url = authorizeUrl(
        provider,
        flow.state,
        await codeChallenge(flow.verifier),
      );
      if (url) return leave(url);
      setLoginError(errorMessage(`auth.${provider}_not_configured`));
    } catch {
      // Sem `sessionStorage` ou sem `crypto.subtle`: o fluxo não tem como voltar.
      setLoginError(errorMessage("internal"));
    }
    setLeaving(false);
  };

  const link = (currentPassword?: string, onError?: (code: string) => void) =>
    run(
      async () => {
        const flow = startOAuthFlow(provider, "link", returnTo, locale);
        const body = {
          state: flow.state,
          code_challenge: await codeChallenge(flow.verifier),
          current_password: currentPassword,
        };
        return provider === "discord"
          ? api.POST("/auth/discord/authorize", { body })
          : api.POST("/auth/google/authorize", { body });
      },
      ({ url }) => leave(url),
      onError,
    );

  const start = (currentPassword?: string, onError?: (code: string) => void) =>
    action === "login" ? void signIn() : link(currentPassword, onError);

  return {
    start,
    pending: pending || leaving,
    error: action === "login" ? loginError : error,
  };
}
