import { Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { LocalDateTime } from "@/shared/ui/local-date-time";
import { Notice } from "@/shared/ui/notice";
import { StructuredList } from "@/shared/ui/structured-list";
import type { TournamentFormState } from "../hooks/use-tournament-form";
import { useValuesSummary } from "../hooks/use-values-summary";
import { checkInOpensNow } from "../lib/form-values";

type Props = { form: TournamentFormState };

type Row =
  | "name"
  | "startsAt"
  | "format"
  | "registration"
  | "champions"
  | "languages";

/**
 * O que vai ser publicado, para conferir antes de "Criar": nome, início,
 * formato, inscrições, campeões e idiomas. Se pela data o check-in já abriu,
 * avisa que o torneio vai direto para ele, sem inscrições.
 */
export function TournamentReview({ form }: Props) {
  const t = useTranslations("TournamentForm.review");
  const tLocales = useTranslations("Locales");
  const summary = useValuesSummary();
  const { values } = form;
  const languages = [
    form.original,
    ...form.others.filter((l) => form.translationOf(l).enabled),
  ];
  // Só existe com o Modal aberto: o "agora" é o da revisão.
  const skipsRegistration = checkInOpensNow(values, new Date());
  const rows: [Row, ReactNode][] = [
    ["name", values.name],
    [
      "startsAt",
      values.startsAt ? (
        <LocalDateTime key="startsAt" value={values.startsAt} />
      ) : (
        ""
      ),
    ],
    ["format", summary.format(values)],
    ["registration", summary.registration(values)],
    ["champions", summary.champions(values)],
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
      {skipsRegistration && <Notice kind="warning">{t("checkInNow")}</Notice>}
    </Stack>
  );
}
