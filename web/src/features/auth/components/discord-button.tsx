"use client";

import { LogoDiscord } from "@carbon/icons-react";
import { Button } from "@carbon/react";
import { useLocale } from "next-intl";
import type { ReactNode } from "react";

import { getPathname } from "@/i18n/navigation";
import { API_URL } from "@/shared/api/url";

type Props = {
  /** Entrar com o Discord ou vinculá-lo à conta logada. */
  action: "login" | "link";
  /** Caminho, sem idioma, para onde a API volta depois. */
  returnTo: string;
  children: ReactNode;
};

/**
 * Leva ao OAuth do Discord na API, que volta para `returnTo` neste idioma.
 * É um link para fora do site (a API), por isso o `Button` do Carbon com
 * `href` (um `<a>`) e não o `ButtonLink`, que é para páginas daqui. Terciário:
 * a outra forma de entrar, ao lado do primário do formulário; o logo é o que
 * a pessoa procura para reconhecer o login de terceiros.
 */
export function DiscordButton({ action, returnTo, children }: Props) {
  const locale = useLocale();
  const back = getPathname({ href: returnTo, locale });
  const href = `${API_URL}/auth/discord/${action}?return_to=${encodeURIComponent(back)}`;

  return (
    <Button href={href} kind="tertiary" renderIcon={LogoDiscord}>
      {children}
    </Button>
  );
}
