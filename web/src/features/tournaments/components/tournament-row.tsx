import { useLocale, useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { RowLink } from "@/shared/ui/row-link";
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
      <RowLink
        href={`/${tenant}/${tournament.slug}`}
        title={localizedName(tournament, locale)}
      >
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
      </RowLink>
    </li>
  );
}
