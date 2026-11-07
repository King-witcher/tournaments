"use client";

import { type ReactNode, Suspense, useEffect, useMemo, useState } from "react";

import { usePathname } from "@/i18n/navigation";
import styles from "./app-shell.module.scss";
import { AppShellContext } from "./app-shell-context";

/**
 * A casca de todas as páginas: o cabeçalho, o conteúdo e o rodapé, mais o
 * estado do menu lateral. Página com `AppSideNav` ganha o recuo dele do `lg`
 * para cima e o botão de menu no cabeçalho abaixo disso; sem ele, nada muda
 * (o CSS confere com `:has`, sem esperar o JS, e a página não pula).
 */
export function AppShell({ children }: { children: ReactNode }) {
  const [navOpen, setNavOpen] = useState(false);
  const value = useMemo(() => ({ navOpen, setNavOpen }), [navOpen]);
  return (
    <AppShellContext value={value}>
      <div className={styles.shell} data-app-shell="">
        {children}
      </div>
      {/* O endereço só existe na requisição: fora do Suspense, a página
          inteira deixaria de ser pré-renderizada. */}
      <Suspense fallback={null}>
        <CloseOnNavigation close={() => setNavOpen(false)} />
      </Suspense>
    </AppShellContext>
  );
}

/** Ir para outra página fecha o menu aberto por cima dela. */
function CloseOnNavigation({ close }: { close: () => void }) {
  const pathname = usePathname();
  // biome-ignore lint/correctness/useExhaustiveDependencies: fecha a cada endereço novo
  useEffect(() => close(), [pathname]);
  return null;
}
