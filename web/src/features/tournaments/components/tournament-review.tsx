import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { StructuredList } from "@/shared/ui/structured-list";
import { useValuesSummary } from "../hooks/use-values-summary";
import { isTdm, valuesFrom } from "../lib/form-values";
import { isTableFormat } from "../lib/tiebreakers";

type Props = {
  /** O rascunho como está salvo. */
  tournament: components["schemas"]["Tournament"];
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
 * O que vai ser publicado, para conferir antes de "Publicar": nome, início,
 * formato, desempate (só nas fases de tabela), quem age pelo time (só nos
 * times), inscrições, mapas, campeões, resultados e idiomas, da configuração
 * salva do rascunho.
 */
export function TournamentReview({ tournament }: Props) {
  const t = useTranslations("TournamentForm.review");
  const tLocales = useTranslations("Locales");
  const summary = useValuesSummary();
  const values = valuesFrom(tournament);
  const languages = [
    tournament.locale,
    ...tournament.translations.map((x) => x.locale),
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
    // O pool salvo é sempre uma lista: os mapas ativos não fazem falta.
    ["veto", summary.veto(values, [])],
    ["champions", summary.champions(values)],
    // A captura obrigatória só liga com capturas na API.
    ["results", summary.results(values, true)],
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
