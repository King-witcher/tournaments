import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { StatusTag } from "@/shared/ui/status-tag";
import { SITUATION_STATUS } from "../lib/situation";

type Schemas = components["schemas"];

type Props = {
  /** A entrada de "Seus torneios"; sem ela (a API falhou), o que o torneio diz. */
  entry: Schemas["MyTournament"] | undefined;
  tournament: Schemas["Tournament"];
};

/**
 * A situação de quem joga depois do início, na etiqueta: na chave, fora por
 * não ter feito check-in, eliminado (com a colocação), desistiu, campeão.
 */
export function BracketSituationTag({ entry, tournament }: Props) {
  const t = useTranslations("Participants");
  const situation: Schemas["Situation"] =
    entry?.situation ??
    (tournament.check_in && !tournament.checked_in
      ? "missed_check_in"
      : "playing");
  const placement = entry?.placement;
  // No empate a decidir, a melhor posição do grupo ("Empate no 3º").
  const place = placement
    ? placement.tied > 1 && situation !== "tie_pending"
      ? t("situation.range", {
          from: placement.rank,
          to: placement.rank + placement.tied - 1,
        })
      : t("situation.rank", { rank: placement.rank })
    : "";
  return (
    <StatusTag status={SITUATION_STATUS[situation]}>
      {t(`situation.${situation}`, { place, stage: entry?.stage ?? "" })}
    </StatusTag>
  );
}
