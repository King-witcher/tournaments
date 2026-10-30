import { Stack } from "@carbon/react";
import type { ReactNode } from "react";

import { Section } from "@/shared/ui/section";
import styles from "./auto-rules-block.module.scss";

type Props = {
  /** Título curto do trecho ("Inscrição e check-in", "Resultado"). */
  title: string;
  /** Parágrafos (`<p>`) e listas (`AutoRulesList`). */
  children: ReactNode;
};

/**
 * Um trecho do "Como funciona" da aba Regras: título (h3) e o texto, na
 * coluna de leitura das regras adicionais.
 */
export function AutoRulesBlock({ title, children }: Props) {
  return (
    <Section level={3} title={title}>
      <Stack gap={4} className={styles.text}>
        {children}
      </Stack>
    </Section>
  );
}
