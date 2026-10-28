import styles from "./user-text.module.scss";

/**
 * Texto escrito por usuário (descrição, regras), sem formatação: as quebras
 * de linha ficam e o resto sai como texto (nada de HTML nem Markdown).
 */
export function UserText({ children }: { children: string }) {
  return <div className={styles.text}>{children}</div>;
}
