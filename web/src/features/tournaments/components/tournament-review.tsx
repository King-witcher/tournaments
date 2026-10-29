import { Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { StructuredList } from "@/shared/ui/structured-list";
import type { TournamentFormState } from "../hooks/use-tournament-form";
import { useValuesSummary } from "../hooks/use-values-summary";
import { StartDateWarning } from "./start-date-warning";

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
  | "languages";

/**
 * O que vai ser publicado, para conferir antes de "Criar": nome, início,
 * formato, inscrições, campeões, veto e idiomas, com o aviso da data de início
 * quando ela não deixa tempo para inscrição e check-in.
 */
export function TournamentReview({ form, maps }: Props) {
  const t = useTranslations("TournamentForm.review");
  const tLocales = useTranslations("Locales");
  const summary = useValuesSummary();
  const { values } = form;
  const languages = [
    form.original,
    ...form.others.filter((l) => form.translationOf(l).enabled),
  ];
  // Só existe com o Modal aberto: o "agora" é o da revisão.
  const now = new Date();
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
    ["languages", languages.map((l) => tLocales(l)).join(", ")],
  ];

  return (
    <Stack gap={5}>
      <StructuredList
        label={t("label")}
        condensed
        rows={rows.map(([key, value]) => ({
          key,
          cells: [t(`labels.${key}`), value],
        }))}
      />
      <StartDateWarning
        startsAt={values.startsAt}
        checkInMinutes={values.checkInMinutes}
        now={now}
      />
    </Stack>
  );
}
