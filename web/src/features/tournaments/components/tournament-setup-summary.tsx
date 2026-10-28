import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { useFormatMinutes } from "@/shared/hooks/use-format-minutes";

type Props = { tournament: components["schemas"]["Tournament"] };

/**
 * Chave, séries, vagas e regras de campeões da fase única, e as janelas de
 * check-in e de contestação.
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
  const rows = [
    [t("labels.bracket"), t(`bracket.${bracket}`)],
    [
      t("labels.series"),
      t(`series.${stage.bracket}`, {
        bestOf: stage.best_of,
        finals: stage.finals_best_of,
      }),
    ],
    [t("labels.slots"), String(tournament.max_participants)],
    [
      t("labels.champions"),
      `${t("bans", { count: rules.bans_per_game })} · ${t(`noRepeat.${rules.no_repeat?.scope ?? "off"}`)}`,
    ],
    [
      t("labels.checkIn"),
      tournament.check_in_minutes == null
        ? t("noCheckIn")
        : t("checkIn", { window: formatMinutes(tournament.check_in_minutes) }),
    ],
    [
      t("labels.contest"),
      t("contest", { window: formatMinutes(tournament.contest_minutes) }),
    ],
  ];

  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-xl font-semibold">{t("title")}</h2>
      <dl className="grid grid-cols-[auto_1fr] gap-x-6 gap-y-1 text-sm">
        {rows.map(([label, value]) => (
          <div key={label} className="contents">
            <dt className="text-muted">{label}</dt>
            <dd>{value}</dd>
          </div>
        ))}
      </dl>
    </section>
  );
}
