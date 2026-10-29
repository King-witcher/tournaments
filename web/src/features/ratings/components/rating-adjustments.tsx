import { useLocale, useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { DataTable } from "@/shared/ui/data-table";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { Section } from "@/shared/ui/section";
import { formatPoints } from "../lib/format-points";

type Props = {
  adjustments: components["schemas"]["RatingAdjustment"][];
};

/**
 * "Ajustes manuais" na página da pessoa, só para a organização: quando,
 * pontos, motivo e quem fez, do mais recente ao mais antigo.
 */
export function RatingAdjustments({ adjustments }: Props) {
  const t = useTranslations("Ratings.adjustments");
  const locale = useLocale();
  return (
    <Section title={t("title")} description={t("description")}>
      <DataTable
        label={t("label")}
        columns={[
          { key: "when", header: t("when"), secondary: true },
          { key: "points", header: t("points") },
          { key: "reason", header: t("reason") },
          { key: "by", header: t("by"), secondary: true },
        ]}
        rows={adjustments.map((a) => ({
          id: String(a.id),
          search: `${a.reason} ${a.created_by?.nickname ?? ""}`,
          cells: {
            when: <LocalDateTime value={a.created_at} />,
            points: formatPoints(a.delta, locale),
            reason: a.reason,
            by: a.created_by?.nickname ?? t("unknownAuthor"),
          },
        }))}
        empty={{ text: t("empty") }}
      />
    </Section>
  );
}
