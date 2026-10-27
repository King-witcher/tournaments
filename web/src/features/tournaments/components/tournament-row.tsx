import { useLocale, useTranslations } from "next-intl";

import { Link } from "@/i18n/navigation";
import type { components } from "@/shared/api/schema";
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
        className="flex items-center justify-between gap-4 py-2 hover:underline"
      >
        <span className="font-medium">{localizedName(tournament, locale)}</span>
        <span className="text-sm opacity-70">{details.join(" · ")}</span>
      </Link>
    </li>
  );
}
