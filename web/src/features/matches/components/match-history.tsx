import { AccordionItem } from "@carbon/react";
import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { MatchEventRow } from "./match-event-row";
import styles from "./match-history.module.scss";

type Props = { detail: components["schemas"]["MatchDetail"] };

/**
 * O bloco fechado "Histórico", no fim da partida: tudo o que aconteceu, com
 * quem e quando, do mais antigo. Vai dentro de um `Accordion`.
 */
export function MatchHistory({ detail }: Props) {
  const t = useTranslations("Match.history");

  return (
    <AccordionItem title={t("title")}>
      {detail.history.length === 0 ? (
        <p className={styles.empty}>{t("empty")}</p>
      ) : (
        <ol className={styles.events}>
          {detail.history.map((event) => (
            <MatchEventRow
              key={`${event.at}:${event.kind}:${event.game}`}
              event={event}
            />
          ))}
        </ol>
      )}
    </AccordionItem>
  );
}
