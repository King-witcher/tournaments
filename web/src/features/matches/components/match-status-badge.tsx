import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Badge } from "@/shared/ui/badge";
import { DeadlineCountdown } from "./deadline-countdown";

type Props = { match: components["schemas"]["BracketMatch"] };

/**
 * Rodapé do cartão: o que a partida espera de quem vê (com o tempo que falta
 * para responder) ou, para os outros, se o resultado está pendente ou
 * contestado. Nada quando não há o que dizer.
 */
export function MatchStatusBadge({ match: m }: Props) {
  const t = useTranslations("Bracket");
  const deadline = m.result?.confirm_by;
  const timed = m.task === "respond" || m.task === "await_opponent";

  let badge = null;
  if (m.task) {
    const urgent = m.task !== "await_opponent" && m.task !== "await_organizer";
    badge = (
      <Badge variant={urgent ? "accent" : "neutral"}>
        {t(`task.${m.task}`)}
        {timed && deadline && (
          <>
            {" · "}
            <DeadlineCountdown until={deadline} />
          </>
        )}
      </Badge>
    );
  } else if (m.result && m.result.status !== "confirmed") {
    badge = (
      <Badge variant={m.result.status === "contested" ? "warning" : "neutral"}>
        {t(`result.${m.result.status}`)}
      </Badge>
    );
  } else if (m.yours && m.state === "ready") {
    badge = <Badge>{t("yours")}</Badge>;
  }
  if (!badge) return <div className="pb-1.5" />;
  return <div className="px-3 pt-0.5 pb-2">{badge}</div>;
}
