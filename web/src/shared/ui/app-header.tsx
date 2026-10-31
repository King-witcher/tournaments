"use client";

import {
  Header,
  HeaderGlobalBar,
  HeaderName,
  SkipToContent,
} from "@carbon/react";
import type { ReactNode } from "react";

import { Link as IntlLink } from "@/i18n/navigation";
import styles from "./app-header.module.scss";
import { Logo } from "./logo";

type Props = {
  /** O nome do site: o do logo que leva ao início, para o leitor de tela. */
  name: string;
  /** Texto do "pular para o conteúdo" (aparece no primeiro Tab). */
  skipLabel: string;
  /** As ações da barra, da esquerda para a direita (idioma, a pessoa). */
  children: ReactNode;
};

/**
 * O cabeçalho de todas as páginas (`Header` do UI Shell do Carbon): pular
 * para o conteúdo, o logo como link para o início e a barra de ações à
 * direita. O conteúdo vai em `AppContent`, logo depois.
 */
export function AppHeader({ name, skipLabel, children }: Props) {
  return (
    <Header aria-label={name}>
      <SkipToContent href="#main-content">{skipLabel}</SkipToContent>
      <HeaderName as={IntlLink} href="/" prefix="" aria-label={name}>
        <Logo />
      </HeaderName>
      <HeaderGlobalBar className={styles.bar}>{children}</HeaderGlobalBar>
    </Header>
  );
}
