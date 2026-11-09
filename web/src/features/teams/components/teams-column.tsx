import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { EmptyState } from "@/shared/ui/empty-state";
import { Section } from "@/shared/ui/section";
import { TEAMS_ID } from "../lib/focus";
import styles from "./teams-column.module.scss";

type Props = {
  count: number;
  /** Ao lado do título ("Sortear times"). */
  action?: ReactNode;
  /** O que dizer sem nenhum time, quando não há `extra`. */
  empty: string;
  /** Os times (`TeamBlock`). */
  children: ReactNode;
  /** Depois dos times, na mesma grade (a área de time novo da edição). */
  extra?: ReactNode;
};

/**
 * A seção dos times: o título com quantos são e os times em grade (uma coluna
 * no celular, duas do md para cima). Sem time e sem `extra`, o estado vazio.
 * Recebe o foco depois de um sorteio.
 */
export function TeamsColumn({ count, action, empty, children, extra }: Props) {
  const t = useTranslations("Team.teams");
  return (
    <Section title={t("title", { count })} action={action}>
      {count === 0 && !extra ? (
        <EmptyState>{empty}</EmptyState>
      ) : (
        <div id={TEAMS_ID} tabIndex={-1} className={styles.grid}>
          {children}
          {extra}
        </div>
      )}
    </Section>
  );
}
