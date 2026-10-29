import { AccordionItem, Stack } from "@carbon/react";
import type { ReactNode } from "react";

import styles from "./form-accordion-item.module.scss";

type Props = {
  /** O nome do grupo de campos. */
  title: ReactNode;
  /** Os valores de agora, numa linha: dá para conferir sem abrir. */
  summary: ReactNode;
  /** O `data-group` dos campos, para o formulário abrir o grupo do campo recusado. */
  group: string;
  open: boolean;
  onToggle: (open: boolean) => void;
  children: ReactNode;
};

/**
 * Um grupo de campos com padrão bom, fechado num item de `Accordion` com o
 * resumo dos valores no título. Aberto pelo formulário quando um campo de
 * dentro é recusado.
 */
export function FormAccordionItem({
  title,
  summary,
  group,
  open,
  onToggle,
  children,
}: Props) {
  return (
    <AccordionItem
      open={open}
      onHeadingClick={({ isOpen }) => onToggle(isOpen)}
      title={
        <span className={styles.title}>
          <span className={styles.name}>{title}</span>
          <span className={styles.summary}>{summary}</span>
        </span>
      }
    >
      <div data-group={group}>
        <Stack gap={6}>{children}</Stack>
      </div>
    </AccordionItem>
  );
}
