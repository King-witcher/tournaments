"use client";

import { createContext, useContext } from "react";

type SearchTransition = {
  /** Uma troca de endereço está no ar (a página nova não chegou). */
  pending: boolean;
  /**
   * Vai para `href` (o mesmo lugar, outro filtro ou página) numa transição,
   * sem rolar; `before` roda dentro dela (o valor otimista do controle).
   */
  navigate: (href: string, before?: () => void) => void;
};

export const SearchTransitionContext = createContext<SearchTransition | null>(
  null,
);

/** A transição dos filtros da aba (`SearchTransition` em volta). */
export function useSearchTransition() {
  const context = useContext(SearchTransitionContext);
  if (!context) throw new Error("useSearchTransition fora de SearchTransition");
  return context;
}
