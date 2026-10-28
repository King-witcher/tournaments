import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { useFormatMinutes } from "@/shared/hooks/use-format-minutes";
import { Section } from "@/shared/ui/section";
import { StructuredList } from "@/shared/ui/structured-list";

type Props = { tournament: components["schemas"]["Tournament"] };

/**
 * O formato, em rótulo e valor: chave, séries, vagas e regras de campeões da
 * fase única, e as janelas de check-in e de contestação.
 */
export function TournamentSetupSummary({ tournament }: Props) {
  const t = useTranslations("Tournament.setup");
  const formatMinutes = useFormatMinutes();
  const [stage] = tournament.stages;
  const rules = tournament.champion_rules;
  const bracket =
    stage.bracket === "double" && stage.grand_final_reset
      ? "doubleReset"
      : stage.bracket;
  const rows: [string, string, string][] = [
    ["bracket", t("labels.bracket"), t(`bracket.${bracket}`)],
    [
      "series",
      t("labels.series"),
      t(`series.${stage.bracket}`, {
        bestOf: stage.best_of,
        finals: stage.finals_best_of,
      }),
    ],
    ["slots", t("labels.slots"), String(tournament.max_participants)],
    [
      "champions",
      t("labels.champions"),
      `${t("bans", { count: rules.bans_per_game })} · ${t(`noRepeat.${rules.no_repeat?.scope ?? "off"}`)}`,
    ],
    [
      "checkIn",
      t("labels.checkIn"),
      tournament.check_in_minutes == null
        ? t("noCheckIn")
        : t("checkIn", { window: formatMinutes(tournament.check_in_minutes) }),
    ],
    [
      "contest",
      t("labels.contest"),
      t("contest", { window: formatMinutes(tournament.contest_minutes) }),
    ],
  ];

  return (
    <Section title={t("title")}>
      <StructuredList
        label={t("title")}
        condensed
        rows={rows.map(([key, label, value]) => ({
          key,
          cells: [label, value],
        }))}
      />
    </Section>
  );
}
