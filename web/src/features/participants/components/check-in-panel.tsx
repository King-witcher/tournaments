import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { CheckInButton } from "./check-in-button";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
  /** Referência para saber se a abertura do check-in já passou. */
  now: Date;
};

/**
 * O check-in para quem vê: quando abre, e no check-in o botão de fazer ou
 * desfazer (só para inscritos). Some sem check-in no torneio.
 */
export function CheckInPanel({ tenant, tournament, now }: Props) {
  const t = useTranslations("Participants.checkIn");
  const opensAt = tournament.check_in_opens_at;
  if (tournament.check_in_minutes == null || !opensAt) return null;

  const hint = (text: ReactNode) => (
    <p className="text-sm opacity-70">{text}</p>
  );
  if (tournament.status === "registration") {
    // Com a abertura no passado e ainda em inscrição, a organização reabriu.
    return new Date(opensAt) <= now
      ? hint(t("suspended"))
      : hint(
          t.rich("opensAt", { date: () => <LocalDateTime value={opensAt} /> }),
        );
  }
  if (tournament.status !== "check_in") return null;
  if (!tournament.registered) return hint(t("openNotRegistered"));

  const done = tournament.checked_in;
  return (
    <div className="flex flex-wrap items-center gap-3">
      <p className={done ? "text-sm font-medium" : "text-sm"}>
        {done ? t("done") : t("open")}
      </p>
      <CheckInButton tenant={tenant} slug={tournament.slug} checkedIn={done} />
    </div>
  );
}
