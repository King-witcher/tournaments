"use client";

import {
  createContext,
  type ReactNode,
  useContext,
  useMemo,
  useState,
} from "react";

export type NavTab = { href: string; label: string };
export type RegisteredTournamentNav = { slug: string; tabs: NavTab[] };

type State = {
  current: RegisteredTournamentNav | null;
  setCurrent: (nav: RegisteredTournamentNav | null) => void;
};

const TournamentNavContext = createContext<State>({
  current: null,
  setCurrent: () => {},
});

/**
 * Liga a página do torneio ao menu lateral da comunidade: a página, que tem o
 * torneio inteiro, diz as seções exatas (`TournamentNavRegistration`); o menu,
 * que só tem a lista, usa as dela até lá. No `layout.tsx` da comunidade, em
 * volta do menu e das páginas.
 */
export function TournamentNavProvider({ children }: { children: ReactNode }) {
  const [current, setCurrent] = useState<RegisteredTournamentNav | null>(null);
  const value = useMemo(() => ({ current, setCurrent }), [current]);
  return <TournamentNavContext value={value}>{children}</TournamentNavContext>;
}

export function useTournamentNav() {
  return useContext(TournamentNavContext);
}
