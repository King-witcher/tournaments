import type { ReactNode } from "react";

import styles from "./legacy-field.module.scss";

type Props = {
  label: string;
  hint?: string;
  children: ReactNode;
};

/**
 * @deprecated Use o campo do Carbon com o rótulo embutido: `TextInput`,
 * `TextArea`, `Select`, `NumberInput`, `DateTimeInput` com `labelText` e
 * `helperText`. Este embrulha um controle antigo num `<label>`.
 */
export function Field({ label, hint, children }: Props) {
  return (
    // biome-ignore lint/a11y/noLabelWithoutControl: o controle chega por `children`, dentro do label.
    <label className={styles.field}>
      <span className={styles.label}>{label}</span>
      {children}
      {hint && <span className={styles.hint}>{hint}</span>}
    </label>
  );
}
