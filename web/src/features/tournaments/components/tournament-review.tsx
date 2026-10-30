import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { useScreenshotSettings } from "@/features/screenshots/hooks/use-screenshot-settings";
import type { components } from "@/shared/api/schema";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { StructuredList } from "@/shared/ui/structured-list";
import type { TournamentFormState } from "../hooks/use-tournament-form";
import { useValuesSummary } from "../hooks/use-values-summary";

type Props = {
  form: TournamentFormState;
  /** Os mapas ativos, para contar o pool padrão. */
  maps: components["schemas"]["Map"][];
};

type Row =
  | "name"
  | "startsAt"
  | "format"
  | "registration"
  | "champions"
  | "veto"
  | "results"
  | "languages";

/**
 * O que vai ser publicado, para conferir antes de "Criar": nome, início,
 * formato, inscrições, campeões, veto, resultados e idiomas.
 */
export function TournamentReview({ form, maps }: Props) {
  const t = useTranslations("TournamentForm.review");
  const tLocales = useTranslations("Locales");
  const summary = useValuesSummary();
  const screenshots = useScreenshotSettings()?.enabled ?? false;
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
    ["registration", summary.registration(values)],
    ["champions", summary.champions(values)],
    ["veto", summary.veto(values, maps)],
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
