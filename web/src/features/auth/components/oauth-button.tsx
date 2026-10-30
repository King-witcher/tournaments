"use client";

import type { ReactNode } from "react";

import { ActionWithNote } from "@/shared/ui/action-with-note";
import { useOAuthStart } from "../hooks/use-oauth-start";
import type { OAuthIntent } from "../lib/oauth-flow";
import type { Provider } from "../lib/providers";
import { ProviderButton } from "./provider-button";

type Props = {
  provider: Provider;
  /** Entrar com o provedor ou vinculá-lo à conta logada (sem senha). */
  action: OAuthIntent;
  /** Caminho, sem idioma, para onde voltar depois. */
  returnTo: string;
  children: ReactNode;
};

/**
 * Leva ao OAuth do provedor (`useOAuthStart`). Provedor desligado (ou, ao
 * vincular, a API recusando: sem sessão ou já vinculado), o erro fica embaixo
 * do botão, sem sair do site. A conta com senha vincula pelo
 * `LinkProviderForm`, que pede a atual.
 */
export function OAuthButton({ provider, action, returnTo, children }: Props) {
  const { start, pending, error } = useOAuthStart(provider, action, returnTo);

  return (
    <ActionWithNote error={error}>
      <ProviderButton
        provider={provider}
        pending={pending}
        onClick={() => start()}
      >
        {children}
      </ProviderButton>
    </ActionWithNote>
  );
}
