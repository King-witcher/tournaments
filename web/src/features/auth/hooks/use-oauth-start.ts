import { useLocale } from "next-intl";
import { useEffect, useState } from "react";

import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import {
  codeChallenge,
  type OAuthIntent,
  startOAuthFlow,
} from "../lib/oauth-flow";
import type { Provider } from "../lib/providers";

/**
 * Abre o OAuth do provedor, com PKCE: `start` abre o fluxo nesta aba (o state
 * e o code verifier ficam só no `sessionStorage` dela), pede à API a URL de
 * autorização com o challenge (e, para vincular numa conta com senha, a
 * atual) e vai para o provedor, que volta para `OAuthCallback` e dali para
 * `returnTo`, neste idioma. A API recusando, `error` traz a mensagem e
 * `onError` recebe o código; a pessoa não sai do site. `pending` continua
 * enquanto o navegador sai para o provedor.
 */
export function useOAuthStart(
  provider: Provider,
  action: OAuthIntent,
  returnTo: string,
) {
  const locale = useLocale();
  const { run, pending, error } = useApiAction();
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

  const start = (currentPassword?: string, onError?: (code: string) => void) =>
    run(
      async () => {
        const flow = startOAuthFlow(provider, action, returnTo, locale);
        const body = {
          intent: action,
          state: flow.state,
          code_challenge: await codeChallenge(flow.verifier),
          current_password: currentPassword,
        };
        return provider === "discord"
          ? api.POST("/auth/discord/authorize", { body })
          : api.POST("/auth/google/authorize", { body });
      },
      ({ url }) => {
        setLeaving(true);
        window.location.assign(url);
      },
      onError,
    );

  return { start, pending: pending || leaving, error };
}
