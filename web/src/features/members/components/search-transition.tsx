"use client";

import { type ReactNode, useTransition } from "react";

import { useRouter } from "@/i18n/navigation";
import { SearchTransitionContext } from "../hooks/use-search-transition";

/**
 * Em volta de uma aba com filtros ou páginas no endereço (Partidas,
 * Confrontos): os controles trocam o endereço por `navigate`, numa transição
 * só, e o conteúdo em `PendingContent` vira esqueleto até a página nova
 * chegar. Sem isso, a troca do mesmo segmento mantém a tela velha sem sinal
 * nenhum, e o controle volta ao valor antigo enquanto espera.
 */
export function SearchTransition({ children }: { children: ReactNode }) {
  const router = useRouter();
  const [pending, start] = useTransition();
  const navigate = (href: string, before?: () => void) =>
    start(() => {
      before?.();
      router.replace(href, { scroll: false });
    });
  return (
    <SearchTransitionContext value={{ pending, navigate }}>
      {children}
    </SearchTransitionContext>
  );
}
