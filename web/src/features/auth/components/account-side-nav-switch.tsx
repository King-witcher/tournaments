"use client";

import type { ReactNode } from "react";

import { usePathname } from "@/i18n/navigation";

/**
 * O menu da conta só nos endereços dela. A navegação do cliente não desmonta
 * o layout da conta ao sair (o Next o guarda escondido, com o `Activity`), e o
 * menu escondido continuaria recuando o conteúdo das outras páginas (a casca
 * confere o menu pelo CSS): fora da conta, nada. O mesmo do menu da
 * comunidade (`TournamentSideNavSwitch`).
 */
export function AccountSideNavSwitch({ children }: { children: ReactNode }) {
  const pathname = usePathname();
  if (pathname !== "/account" && !pathname.startsWith("/account/")) {
    return null;
  }
  return children;
}
