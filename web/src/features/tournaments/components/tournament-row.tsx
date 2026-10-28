import { useLocale, useTranslations } from "next-intl";

import { Link } from "@/i18n/navigation";
import type { components } from "@/shared/api/schema";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { checkInPending } from "../lib/check-in";
import { localizedName } from "../lib/localize";

type Props = {
  tenant: string;
  tournament: components["schemas"]["TournamentSummary"];
};

export function TournamentRow({ tenant, tournament }: Props) {
  const t = useTranslations("Tournaments");
  const tStatus = useTranslations("TournamentStatus");
  const tVisibility = useTranslations("Visibility");
  const locale = useLocale();
  const details = [
    tStatus(tournament.status),
    tournament.visibility === "private" && tVisibility("private"),
    tournament.registered && t("registered"),
    t("participants", {
      count: tournament.participants,
      max: tournament.max_participants,
    }),
  ].filter(Boolean);

  return (
    <li>
      <Link
        href={`/${tenant}/${tournament.slug}`}
        className="flex flex-col gap-0.5 py-2 hover:underline sm:flex-row sm:items-center sm:justify-between sm:gap-4"
      >
        <span className="font-medium">{localizedName(tournament, locale)}</span>
        <span className="text-sm opacity-70 sm:text-right">
          {tournament.starts_at && (
            <>
              <LocalDateTime value={tournament.starts_at} />
              {" · "}
            </>
          )}
          {details.join(" · ")}
          {checkInPending(tournament) && (
            <span className="font-medium text-red-700 dark:text-red-400">
              {" · "}
              {t("checkInPending")}
            </span>
          )}
        </span>
      </Link>
    </li>
  );
}
