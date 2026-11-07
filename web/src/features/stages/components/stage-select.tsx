import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { finalAvailable, startedStages } from "../lib/stages";
import { StageSwitcher } from "./stage-switcher";

type Props = {
  tournament: Pick<components["schemas"]["Tournament"], "status" | "stages">;
  /** A Classificação ganha a opção "Final" com o torneio encerrado. */
  view: "bracket" | "standings";
  /** A fase mostrada: o número ou `final`. */
  value: number | "final";
};

/**
 * O seletor de fase das abas Chave e Classificação: uma opção por fase que
 * começou (e, na Classificação, a final do torneio encerrado). Com uma opção
 * só, nada aparece: o torneio de uma fase fica como sempre foi.
 */
export function StageSelect({ tournament, view, value }: Props) {
  const t = useTranslations("Stages");
  const options = [
    ...startedStages(tournament).map((stage) => ({
      value: String(stage.number),
      text: t("stageOption", { number: stage.number }),
    })),
    ...(view === "standings" && finalAvailable(tournament)
      ? [{ value: "final", text: t("finalOption") }]
      : []),
  ];
  if (options.length < 2) return null;
  return (
    <StageSwitcher
      label={t("switcher")}
      options={options}
      value={String(value)}
    />
  );
}
