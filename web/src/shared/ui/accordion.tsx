import { AccordionItem, Accordion as CarbonAccordion } from "@carbon/react";
import type { ReactNode } from "react";

type Props = {
  /** O que abre (o nome do bloco). */
  title: ReactNode;
  children: ReactNode;
  /** Começa aberto; o padrão é fechado (o bloco é secundário). */
  open?: boolean;
};

/**
 * Um bloco que abre e fecha (`Accordion` do Carbon com um item): o que é
 * secundário e ocuparia a tela toda (encerrados, histórico, organização da
 * partida). Vários blocos seguidos: use `Accordion`/`AccordionItem` do
 * `@carbon/react` direto.
 */
export function Accordion({ title, children, open = false }: Props) {
  return (
    <CarbonAccordion align="start">
      <AccordionItem title={title} open={open}>
        {children}
      </AccordionItem>
    </CarbonAccordion>
  );
}
