"use client";

import { LogoDiscord } from "@carbon/icons-react";
import { useLocale } from "next-intl";
import type { ReactNode } from "react";

import { getPathname } from "@/i18n/navigation";
import { API_URL } from "@/shared/api/url";
import { ButtonLink } from "@/shared/ui/button-link";
import type { Provider } from "../lib/providers";
import { GoogleLogo } from "./google-logo";

type Props = {
  provider: Provider;
  /** Entrar com o provedor ou vinculá-lo à conta logada. */
  action: "login" | "link";
  /** Caminho, sem idioma, para onde a API volta depois. */
  returnTo: string;
  children: ReactNode;
};

const LOGO = { discord: LogoDiscord, google: GoogleLogo };

/**
 * Leva ao OAuth do provedor na API, que volta para `returnTo` neste idioma.
 * É um link para fora do site (a API), por isso o `ButtonLink` `external`.
 * Terciário: a outra forma de entrar, ao lado do primário do formulário; o
 * logo é o que a pessoa procura para reconhecer o login de terceiros.
 */
export function OAuthButton({ provider, action, returnTo, children }: Props) {
  const locale = useLocale();
  const back = getPathname({ href: returnTo, locale });
  const href = `${API_URL}/auth/${provider}/${action}?return_to=${encodeURIComponent(back)}`;

  return (
    <ButtonLink
      external
      href={href}
      kind="tertiary"
      renderIcon={LOGO[provider]}
    >
      {children}
    </ButtonLink>
  );
}
