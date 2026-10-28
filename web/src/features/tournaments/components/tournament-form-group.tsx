import { FormGroup, Stack } from "@carbon/react";
import type { ReactNode } from "react";

import styles from "./tournament-form-group.module.scss";

type Props = {
  /** O nome da seção (vira a `legend`). */
  legend: ReactNode;
  /** Uma frase sobre a seção, antes dos campos. */
  description?: ReactNode;
  children: ReactNode;
};

/**
 * Uma seção do formulário de torneio: `FormGroup` com a legenda no tamanho de
 * título de bloco e os campos empilhados com o espaço do Carbon.
 */
export function TournamentFormGroup({ legend, description, children }: Props) {
  return (
    <FormGroup legendText={<span className={styles.legend}>{legend}</span>}>
      <Stack gap={6}>
        {description && <p className={styles.description}>{description}</p>}
        {children}
      </Stack>
    </FormGroup>
  );
}
