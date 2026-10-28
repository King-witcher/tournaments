import { FormGroup, Stack } from "@carbon/react";
import type { ReactNode } from "react";

import styles from "./form-section.module.scss";

type Props = {
  /** O nome da seção (vira a `legend`). */
  legend: ReactNode;
  /** Uma frase sobre a seção, antes dos campos. */
  description?: ReactNode;
  children: ReactNode;
};

/**
 * Uma seção de formulário: `FormGroup` do Carbon com a legenda no tamanho de
 * título de bloco (`heading-02`) e os campos empilhados com `Stack gap={6}`.
 * Formulário com mais de um grupo de campos usa uma por grupo (um `Stack
 * gap={7}` entre elas); com um grupo só, os campos vão direto no `Stack`.
 */
export function FormSection({ legend, description, children }: Props) {
  return (
    <FormGroup legendText={<span className={styles.legend}>{legend}</span>}>
      <Stack gap={6}>
        {description && <p className={styles.description}>{description}</p>}
        {children}
      </Stack>
    </FormGroup>
  );
}
