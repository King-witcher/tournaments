import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { StructuredList } from "@/shared/ui/structured-list";
import { useStageEntry } from "../hooks/use-stage-entry";
import { useValuesSummary } from "../hooks/use-values-summary";
import { isTdm, valuesFrom } from "../lib/form-values";
import { isTableFormat } from "../lib/tiebreakers";

type Props = {
  /** O rascunho como está salvo. */
  tournament: components["schemas"]["Tournament"];
};

type Row = { key: string; label: ReactNode; value: ReactNode };

/** As linhas com um rótulo só. */
type RowKey =
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
 * salva do rascunho. Com mais de uma fase, o formato é a modalidade e as
 * vagas, e cada fase tem a sua linha ("Fase 2") e, da segunda em diante, a
 * entrada dela.
 */
export function TournamentReview({ tournament }: Props) {
  const t = useTranslations("TournamentForm.review");
  const tLocales = useTranslations("Locales");
  const summary = useValuesSummary();
  const entry = useStageEntry();
  const values = valuesFrom(tournament);
  const languages = [
    tournament.locale,
    ...tournament.translations.map((x) => x.locale),
  ];
  const several = values.stages.length > 1;
  const row = (key: RowKey, value: ReactNode): Row => ({
    key,
    label: t(`labels.${key}`),
    value,
  });
  const [first] = values.stages;
  const rows: Row[] = [
    row("name", values.name),
    row(
      "startsAt",
      values.startsAt ? (
        <LocalDateTime key="startsAt" value={values.startsAt} />
      ) : (
        t("noDate")
      ),
    ),
    ...(several
      ? [
          row("format", summary.mode(values)),
          ...values.stages.flatMap((stage, i): Row[] => [
            {
              key: `stage-${i}`,
              label: t("labels.stage", { number: i + 1 }),
              value: [
                summary.stage(stage, values),
                isTableFormat(stage.format) &&
                  `${t("labels.tiebreakers")}: ${summary.tiebreakers(stage)}`,
              ]
                .filter(Boolean)
                .join(" · "),
            },
            ...(i > 0
              ? [
                  {
                    key: `entry-${i}`,
                    label: t("labels.entry"),
                    value: entry(stage.advance, stage.checkIn, i + 1).join(" "),
                  },
                ]
              : []),
          ]),
        ]
      : [
          row("format", summary.format(values)),
          ...(isTableFormat(first.format)
            ? [row("tiebreakers", summary.tiebreakers(first))]
            : []),
        ]),
    ...(isTdm(values) ? [row("teamRules", summary.teamRules(values))] : []),
    row("registration", summary.registration(values)),
    // O pool salvo é sempre uma lista: os mapas ativos não fazem falta.
    row("veto", summary.veto(values, [])),
    row("champions", summary.champions(values)),
    // A captura obrigatória só liga com capturas na API.
    row("results", summary.results(values, true)),
    row("languages", languages.map((l) => tLocales(l)).join(", ")),
  ];

  return (
    <StructuredList
      label={t("label")}
      condensed
      rows={rows.map(({ key, label, value }) => ({
        key,
        cells: [label, value],
      }))}
    />
  );
}
