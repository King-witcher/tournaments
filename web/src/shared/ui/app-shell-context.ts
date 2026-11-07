"use client";

import { createContext, useContext } from "react";

type AppShellState = {
  /** O menu lateral aberto por cima da página (abaixo do `lg`). */
  navOpen: boolean;
  setNavOpen: (open: boolean) => void;
};

export const AppShellContext = createContext<AppShellState>({
  navOpen: false,
  setNavOpen: () => {},
});

/** O estado do menu lateral: o botão do cabeçalho abre, o `AppSideNav` lê. */
export function useAppShell() {
  return useContext(AppShellContext);
}
