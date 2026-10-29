import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
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
 * O estado do time como etiqueta: antes de iniciar, completo, faltando
 * jogadores ou com check-in pendente; depois, desistiu ou fora (ninguém dele
 * na comunidade). Sem nada a dizer, nada.
 */
export function TeamStateTag({ team, formation, checkIn, started }: Props) {
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
      return (
        <StatusTag status="confirmed" size="sm">
          {t("complete")}
        </StatusTag>
      );
    case "missing":
      return (
        <StatusTag status="waiting" size="sm">
          {t("missing", {
            count: formation.team_size - team.members.length,
          })}
        </StatusTag>
      );
    case "checkInPending":
      return (
        <StatusTag status="check_in" size="sm">
          {t("checkInPending")}
        </StatusTag>
      );
  }
}
