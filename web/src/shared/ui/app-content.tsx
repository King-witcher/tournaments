import { Content } from "@carbon/react";
import type { ReactNode } from "react";

import styles from "./app-content.module.scss";

/**
 * A área de conteúdo abaixo do `AppHeader` (`Content` do Carbon, o `<main>`
 * para onde o "pular para o conteúdo" leva). Cada página põe o `Page` dentro.
 */
export function AppContent({ children }: { children: ReactNode }) {
  return (
    <Content id="main-content" className={styles.content}>
      {children}
    </Content>
  );
}
