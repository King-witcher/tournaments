"use client";

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

/** Navega para o OAuth do Discord na API, que volta para `returnTo` neste idioma. */
export function DiscordButton({ action, returnTo, children }: Props) {
  const locale = useLocale();
  const back = getPathname({ href: returnTo, locale });
  const href = `${API_URL}/auth/discord/${action}?return_to=${encodeURIComponent(back)}`;

  return (
    <a
      href={href}
      className="self-start rounded-md bg-[#5865F2] px-3 py-1.5 text-sm font-medium text-white hover:bg-[#4752C4]"
    >
      {children}
    </a>
  );
}
