import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { EmptyState } from "@/shared/ui/empty-state";
import { Section } from "@/shared/ui/section";
import { TEAMS_ID } from "../lib/focus";
import styles from "./teams-column.module.scss";

type Props = {
  count: number;
  /** Uma frase embaixo do título (na edição, como mexer). */
  description?: ReactNode;
  /** Ao lado do título ("Formar times"). */
  action?: ReactNode;
  /** O que dizer sem nenhum time. */
  empty: string;
  /** Os times (`TeamBlock`). */
  children: ReactNode;
  /** Depois dos times, na mesma grade (a área de time novo da edição). */
  extra?: ReactNode;
};

/**
 * A seção dos times: o título com quantos são e os times em grade (uma
 * coluna no celular, duas do md para cima). Sem time, o estado vazio, e a
 * grade só aparece se houver `extra`. Recebe o foco depois de um sorteio.
 */
export function TeamsColumn({
  count,
  description,
  action,
  empty,
  children,
  extra,
}: Props) {
  const t = useTranslations("Team.teams");
  return (
    <Section
      title={t("title", { count })}
      description={description}
      action={action}
    >
      {count === 0 && <EmptyState>{empty}</EmptyState>}
      {(count > 0 || extra) && (
        <div id={TEAMS_ID} tabIndex={-1} className={styles.grid}>
          {children}
          {extra}
        </div>
      )}
    </Section>
  );
}
