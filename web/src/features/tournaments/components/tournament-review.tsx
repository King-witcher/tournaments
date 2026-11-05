import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { StructuredList } from "@/shared/ui/structured-list";
import type { TournamentFormState } from "../hooks/use-tournament-form";
import { useValuesSummary } from "../hooks/use-values-summary";
import { isTdm } from "../lib/form-values";
import { isTableFormat } from "../lib/tiebreakers";

type Props = {
  form: TournamentFormState;
  /** Os mapas ativos, para contar o pool padrão. */
  maps: components["schemas"]["Map"][];
  /** Se esta API tem capturas de tela. */
  screenshots: boolean;
};

type Row =
  | "name"
  | "startsAt"
  | "format"
  | "tiebreakers"
  | "teamRules"
  | "registration"
  | "veto"
  | "champions"
  | "results"
  | "languages";

/**
 * O que vai ser publicado, para conferir antes de "Criar": nome, início,
 * formato, desempate (só nas fases de tabela), quem age pelo time (só nos
 * times), inscrições, mapas, campeões, resultados e idiomas.
 */
export function TournamentReview({ form, maps, screenshots }: Props) {
  const t = useTranslations("TournamentForm.review");
  const tLocales = useTranslations("Locales");
  const summary = useValuesSummary();
  const { values } = form;
  const languages = [
    form.original,
    ...form.others.filter((l) => form.translationOf(l).enabled),
  ];
  const rows: [Row, ReactNode][] = [
    ["name", values.name],
    [
      "startsAt",
      values.startsAt ? (
        <LocalDateTime key="startsAt" value={values.startsAt} />
      ) : (
        t("noDate")
      ),
    ],
    ["format", summary.format(values)],
    ...(isTableFormat(values.stageFormat)
      ? ([["tiebreakers", summary.tiebreakers(values)]] as [Row, ReactNode][])
      : []),
    ...(isTdm(values)
      ? ([["teamRules", summary.teamRules(values)]] as [Row, ReactNode][])
      : []),
    ["registration", summary.registration(values)],
    ["veto", summary.veto(values, maps)],
    ["champions", summary.champions(values)],
    ["results", summary.results(values, screenshots)],
    ["languages", languages.map((l) => tLocales(l)).join(", ")],
  ];

  return (
    <StructuredList
      label={t("label")}
      condensed
      rows={rows.map(([key, value]) => ({
        key,
        cells: [t(`labels.${key}`), value],
      }))}
    />
  );
}
