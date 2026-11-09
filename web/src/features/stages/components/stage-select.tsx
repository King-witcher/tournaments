import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { finalAvailable, startedStages } from "../lib/stages";
import { StageTabs } from "./stage-tabs";

type Props = {
  tournament: Pick<components["schemas"]["Tournament"], "status" | "stages">;
  /** A Classificação ganha a opção "Final" com o torneio encerrado. */
  view: "bracket" | "standings";
  /** A fase mostrada: o número ou `final`. */
  value: number | "final";
  /** O que a opção mostrada desenha. */
  children: ReactNode;
};

/**
 * As abas de fase da Chave e da Classificação: uma por fase que começou e,
 * primeiro, na Classificação do torneio encerrado, a final. Com uma opção só,
 * só o conteúdo: o torneio de uma fase fica como sempre foi.
 */
export function StageSelect({ tournament, view, value, children }: Props) {
  const t = useTranslations("Stages");
  const stages = startedStages(tournament).map((stage) => ({
    value: String(stage.number),
    text: t("stageOption", { number: stage.number }),
  }));
  const options =
    view === "standings" && finalAvailable(tournament)
      ? [{ value: "final", text: t("finalOption") }, ...stages]
      : stages;
  if (options.length < 2) return children;
  return (
    <StageTabs label={t("switcher")} options={options} value={String(value)}>
      {children}
    </StageTabs>
  );
}
