import { useCallback, useSyncExternalStore } from "react";

import { THEME_KEY, type ThemeChoice } from "@/shared/lib/theme";

const listeners = new Set<() => void>();

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

function read(): ThemeChoice {
  const theme = document.documentElement.dataset.theme;
  return theme === "light" || theme === "dark" ? theme : "system";
}

/**
 * A escolha de tema de quem usa: `system` (o do navegador), `light` ou
 * `dark`. Fica no `localStorage` e em `data-theme` no `<html>` (o script de
 * `shared/lib/theme.ts` a aplica antes da pintura). `system` no servidor.
 */
export function useTheme() {
  const theme = useSyncExternalStore(subscribe, read, () => "system" as const);

  const setTheme = useCallback((choice: ThemeChoice) => {
    const root = document.documentElement;
    if (choice === "system") delete root.dataset.theme;
    else root.dataset.theme = choice;
    try {
      if (choice === "system") localStorage.removeItem(THEME_KEY);
      else localStorage.setItem(THEME_KEY, choice);
    } catch {
      // Sem armazenamento (janela privada): vale só até fechar a página.
    }
    for (const listener of listeners) listener();
  }, []);

  return { theme, setTheme };
}
