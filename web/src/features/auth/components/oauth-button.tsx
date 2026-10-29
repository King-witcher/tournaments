"use client";

import { LogoDiscord } from "@carbon/icons-react";
import { useLocale } from "next-intl";
import { type ReactNode, useEffect, useState } from "react";

import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button } from "@/shared/ui/button";
import {
  codeChallenge,
  type OAuthIntent,
  startOAuthFlow,
} from "../lib/oauth-flow";
import type { Provider } from "../lib/providers";
import { GoogleLogo } from "./google-logo";

type Props = {
  provider: Provider;
  /** Entrar com o provedor ou vinculá-lo à conta logada. */
  action: OAuthIntent;
  /** Caminho, sem idioma, para onde voltar depois. */
  returnTo: string;
  children: ReactNode;
};

const LOGO = { discord: LogoDiscord, google: GoogleLogo };

/**
 * Leva ao OAuth do provedor, com PKCE: o clique abre o fluxo nesta aba (o
 * state e o code verifier ficam só no `sessionStorage` dela), pede à API a
 * URL de autorização com o challenge e vai para o provedor, que volta para
 * `OAuthCallback` e dali para `returnTo`, neste idioma. A API recusando
 * (provedor desligado, ou, ao vincular, sem sessão ou já vinculado), o erro
 * fica embaixo do botão, sem sair do site. Terciário: a outra forma de
 * entrar, ao lado do primário do formulário; o logo é o que a pessoa procura
 * para reconhecer o login de terceiros.
 */
export function OAuthButton({ provider, action, returnTo, children }: Props) {
  const locale = useLocale();
  const { run, pending, error } = useApiAction();
  // Enquanto o navegador sai para o provedor, o botão continua pendente.
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

  const start = () =>
    run(
      async () => {
        const flow = startOAuthFlow(provider, action, returnTo, locale);
        const body = {
          intent: action,
          state: flow.state,
          code_challenge: await codeChallenge(flow.verifier),
        };
        return provider === "discord"
          ? api.POST("/auth/discord/authorize", { body })
          : api.POST("/auth/google/authorize", { body });
      },
      ({ url }) => {
        setLeaving(true);
        window.location.assign(url);
      },
    );

  return (
    <ActionWithNote error={error}>
      <Button
        kind="tertiary"
        renderIcon={LOGO[provider]}
        pending={pending || leaving}
        onClick={start}
      >
        {children}
      </Button>
    </ActionWithNote>
  );
}
