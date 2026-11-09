import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { IconIndicator } from "@/shared/ui/icon-indicator";
import { StatusTag } from "@/shared/ui/status-tag";
import { type Formation, teamState } from "../lib/formation";
import { teamLeft } from "../lib/team-name";

type Props = {
  team: components["schemas"]["Team"];
  formation: Formation;
  /** O check-in conta para o time estar completo (`checkInShown`). */
  checkIn: boolean;
  /** Já começou: completo ou não deixa de importar; só desistência e saída. */
  started: boolean;
};

/**
 * O estado do time no cabeçalho dele, só quando há o que dizer: antes de
 * iniciar, o que falta (indicador de atenção do Carbon, não vermelho: é
 * trabalho em andamento); desistência e saída (etiqueta, na cor de cancelado).
 * O time completo não leva nada: é o caso esperado.
 */
export function TeamState({ team, formation, checkIn, started }: Props) {
  const t = useTranslations("Team.state");
  if (team.withdrawn) {
    return (
      <StatusTag status="cancelled" size="sm">
        {t("withdrawn")}
      </StatusTag>
    );
  }
  if (teamLeft(team)) {
    return (
      <StatusTag status="cancelled" size="sm">
        {t("out")}
      </StatusTag>
    );
  }
  if (started) return null;
  switch (teamState(team, formation, checkIn)) {
    case "complete":
      return null;
    case "missing":
      return (
        <IconIndicator
          kind="caution-minor"
          label={t("missing", {
            count: formation.team_size - team.members.length,
          })}
        />
      );
    case "checkInPending":
      return <IconIndicator kind="caution-minor" label={t("checkInPending")} />;
  }
}
