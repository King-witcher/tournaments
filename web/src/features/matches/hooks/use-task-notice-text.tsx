import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { LocalDayTime } from "@/shared/ui/local-day-time";
import type { ActionableTask } from "../lib/status";

type Detail = components["schemas"]["TaskDetail"];

/**
 * Os textos do aviso de uma série que espera quem vê (a home e "Séries que
 * esperam você"). `title`: a situação, com quem agiu ("Partida 2 enviada por
 * P4"); `opponent` é o nome do adversário, quando quem vê joga. Sem o
 * detalhe, ou sem o nome de que a frase precisa, sai a frase sem ele. `note`:
 * o que fazer quando o título não diz (o prazo da resposta, o aceite da
 * contestação), ou nada. Serve a Server e a Client Components.
 */
export function useTaskNoticeText() {
  const t = useTranslations("Bracket.pending");
  const tTask = useTranslations("Bracket.task");

  const title = (
    task: ActionableTask,
    detail: Detail | null | undefined,
    opponent?: string,
  ) => {
    if (!detail) return tTask(task);
    const { game, contested_by: player, reported_by: reporter } = detail;
    switch (task) {
      case "report":
        return t("notice.report", { game });
      case "respond":
        return opponent
          ? t("notice.respond", { game, opponent })
          : tTask("respond");
      case "accept_contest":
        return player
          ? t("notice.accept_contest", { player })
          : t("notice.accept_contest_unknown");
      case "ready":
        return detail.opponent_ready && opponent
          ? t("notice.ready", { opponent })
          : t("notice.ready_first");
      case "veto":
        return t(`notice.${detail.veto_step ?? "veto"}`);
      case "resolve":
        if (player && reporter)
          return t("notice.resolve", { player, reporter });
        return player
          ? t("notice.resolve_game", { player, game })
          : t("notice.resolve_unknown", { game });
    }
  };

  const note = (task: ActionableTask, detail: Detail | null | undefined) => {
    const deadline = detail?.confirm_by;
    if (task === "respond" && deadline)
      return t.rich("deadline", {
        deadline: () => <LocalDayTime value={deadline} />,
      });
    if (task === "accept_contest") return t("acceptContest");
    return undefined;
  };

  return { title, note };
}
