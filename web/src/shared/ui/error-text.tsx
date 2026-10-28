import type { ReactNode } from "react";

import styles from "./error-text.module.scss";

/**
 * Erro curto, junto do controle que falhou (um botão de linha que chamou a
 * API). Quebra de linha sempre, mesmo dentro de uma célula de tabela que não
 * quebra. Em campo de formulário, use o `invalidText` do próprio campo.
 */
export function ErrorText({ children }: { children: ReactNode }) {
  return (
    <p role="alert" className={styles.error}>
      {children}
    </p>
  );
}
