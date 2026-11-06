import type { ReactNode } from "react";

import { AccordionItem } from "@/shared/ui/carbon/accordion";
import { Stack } from "@/shared/ui/carbon/stack";
import styles from "./form-accordion-item.module.scss";

type Props = {
  /** O nome do grupo de campos. */
  title: ReactNode;
  /** Os valores, numa linha: dá para conferir sem abrir. */
  summary: ReactNode;
  /** Ao lado do nome, o estado do grupo (mudança não salva, só leitura). */
  status?: ReactNode;
  /** O `data-group` dos campos, para o formulário abrir o grupo do campo recusado. */
  group?: string;
  /** O `id` do título (dentro do botão do item), alvo de um "Ir para". */
  id?: string;
  open: boolean;
  onToggle: (open: boolean) => void;
  children: ReactNode;
};

/**
 * Um grupo de campos fechado num item de `Accordion`, com o resumo dos
 * valores no título: os grupos com padrão bom da sala de veto e as seções da
 * aba Configurações do torneio. Aberto pelo formulário quando um campo de
 * dentro é recusado.
 */
export function FormAccordionItem({
  title,
  summary,
  status,
  group,
  id,
  open,
  onToggle,
  children,
}: Props) {
  return (
    <AccordionItem
      open={open}
      onHeadingClick={({ isOpen }) => onToggle(isOpen)}
      title={
        <span id={id} className={styles.title}>
          <span className={styles.head}>
            <span className={styles.name}>{title}</span>
            {status}
          </span>
          <span className={styles.summary}>{summary}</span>
        </span>
      }
    >
      <div data-group={group} className={styles.group}>
        <Stack gap={6}>{children}</Stack>
      </div>
    </AccordionItem>
  );
}
