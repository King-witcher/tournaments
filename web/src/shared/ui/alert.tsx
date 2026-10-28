import type { ReactNode } from "react";

import styles from "./error-text.module.scss";
import { Notice } from "./notice";

/**
 * @deprecated Use `Notice kind="error"` (erro da página) ou `FormError` (erro
 * geral de formulário, acima dos botões).
 */
export function Alert({ children }: { children: ReactNode }) {
  return <Notice kind="error">{children}</Notice>;
}

/**
 * Erro curto, junto do controle que falhou (um botão de linha que chamou a
 * API). Em campo de formulário, use o `invalidText` do próprio campo.
 */
export function ErrorText({
  children,
}: {
  children: ReactNode;
  /** @deprecated Não muda mais nada: o texto tem um tamanho só. */
  size?: "sm" | "xs";
}) {
  return (
    <p role="alert" className={styles.error}>
      {children}
    </p>
  );
}
