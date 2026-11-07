import type { ReactNode } from "react";

import { Stack } from "@/shared/ui/carbon/stack";
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
 * coluna de leitura das regras da organização.
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
