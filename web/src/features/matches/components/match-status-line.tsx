import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { useSideNames } from "../hooks/use-side-names";
import { SCORE_ID } from "../lib/match-target";
import { taskTag, waitingKey } from "../lib/status";
import styles from "./match-status-line.module.scss";

type Props = { detail: components["schemas"]["MatchDetail"] };

/**
 * A linha embaixo do cabeçalho da série, quando ele não basta: decidida,
 * como e quando (no W.O., quem venceu, que o título não mostra; no empate, sem
 * ninguém); sem lados
 * ainda, sem adversário ou sem precisar, o porquê. Enquanto a série corre, o
 * que ela espera de outro ("Aguardando o adversário"), em texto; a ação de
 * quem vê é o passo em que a página abre, e a contestação, a etiqueta do
 * cabeçalho.
 */
export function MatchStatusLine({ detail }: Props) {
  const t = useTranslations("Match");
  const tBracket = useTranslations("Bracket");
  const names = useSideNames(detail);
  const s = detail.summary;
  const r = s.result;
  let line: ReactNode = null;
  if (s.state === "completed" && r) {
    const time = () => <LocalDateTime value={r.confirmed_at} />;
    if (r.winner == null) {
      line = t.rich("drawnAt", { time });
    } else if (r.walkover) {
      line = t.rich(
        r.confirmed_via === "withdrawal"
          ? "walkover.withdrawal"
          : "walkover.organizer",
        { name: names[r.winner], time },
      );
    } else {
      line = t.rich(`confirmedVia.${r.confirmed_via}`, { time });
    }
  } else if (s.state !== "ready") {
    line = t(`state.${s.state}`);
  } else if (!(s.task && taskTag(s.task) === "contested")) {
    const waiting = waitingKey(s);
    if (waiting) line = tBracket(waiting);
  }
  if (line == null) return null;
  // tabIndex -1: recebe o foco depois do W.O., que fecha o bloco da
  // organização com o botão usado.
  return (
    <p id={SCORE_ID} tabIndex={-1} className={styles.line}>
      {line}
    </p>
  );
}
