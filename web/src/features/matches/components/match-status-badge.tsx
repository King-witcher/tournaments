import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Badge } from "@/shared/ui/badge";

type Props = { match: components["schemas"]["BracketMatch"] };

/**
 * Rodapé do cartão: o que a partida espera de quem vê ou, para os outros, se
 * há mapa esperando confirmação ou contestado. Nada quando não há o que
 * dizer.
 */
export function MatchStatusBadge({ match: m }: Props) {
  const t = useTranslations("Bracket");

  let badge = null;
  if (m.task) {
    const urgent = m.task !== "await_opponent" && m.task !== "await_organizer";
    badge = (
      <Badge variant={urgent ? "accent" : "neutral"}>
        {t(`task.${m.task}`)}
      </Badge>
    );
  } else if (m.contested_maps > 0) {
    badge = <Badge variant="warning">{t("result.contested")}</Badge>;
  } else if (m.pending_maps > 0) {
    badge = <Badge>{t("result.pending")}</Badge>;
  } else if (m.yours && m.state === "ready") {
    badge = <Badge>{t("yours")}</Badge>;
  }
  if (!badge) return <div className="pb-1.5" />;
  return <div className="px-3 pt-0.5 pb-2">{badge}</div>;
}
