"use client";

import type { ReactNode } from "react";

import { Link as IntlLink } from "@/i18n/navigation";
import styles from "./app-header.module.scss";
import { useAppShell } from "./app-shell-context";
import {
  Header,
  HeaderGlobalBar,
  HeaderMenuButton,
  HeaderName,
  SkipToContent,
} from "./carbon/ui-shell";
import { Logo } from "./logo";

type Props = {
  /** O nome do site: o do logo que leva ao início, para o leitor de tela. */
  name: string;
  /** Texto do "pular para o conteúdo" (aparece no primeiro Tab). */
  skipLabel: string;
  /**
   * Textos do botão do menu lateral (aparece só em página com `AppSideNav`).
   * Sem eles, nem o botão.
   */
  menuLabels?: { open: string; close: string };
  /** As ações da barra, da esquerda para a direita (comunidades, a pessoa). */
  children: ReactNode;
};

/**
 * O cabeçalho de todas as páginas (`Header` do UI Shell do Carbon): pular
 * para o conteúdo, o botão do menu lateral (abaixo do `lg`, só em página
 * com `AppSideNav`), o logo como link para o início e a barra de ações à
 * direita. O conteúdo vai em `AppContent`, logo depois.
 */
export function AppHeader({ name, skipLabel, menuLabels, children }: Props) {
  const { navOpen, setNavOpen } = useAppShell();
  return (
    <Header aria-label={name}>
      <SkipToContent href="#main-content">{skipLabel}</SkipToContent>
      {menuLabels && (
        <HeaderMenuButton
          aria-label={navOpen ? menuLabels.close : menuLabels.open}
          aria-expanded={navOpen}
          isActive={navOpen}
          onClick={() => setNavOpen(!navOpen)}
          className={styles.menuButton}
        />
      )}
      <HeaderName as={IntlLink} href="/" prefix="" aria-label={name}>
        <Logo />
      </HeaderName>
      <HeaderGlobalBar className={styles.bar}>{children}</HeaderGlobalBar>
    </Header>
  );
}
