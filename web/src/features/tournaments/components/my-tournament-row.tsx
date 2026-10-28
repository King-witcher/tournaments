import { useLocale, useTranslations } from "next-intl";

import { NextMatchLink } from "@/features/matches/components/next-match-link";
import type { components } from "@/shared/api/schema";
import { Badge } from "@/shared/ui/badge";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { RowLink } from "@/shared/ui/row-link";
import { localizedName } from "../lib/localize";

type Schemas = components["schemas"];

/** Cor da etiqueta: o que pede ação (ou já perdeu a vaga) se destaca. */
const BADGE: Record<
  Schemas["Situation"],
  "neutral" | "accent" | "warning" | "success"
> = {
  registered: "neutral",
  check_in_pending: "warning",
  checked_in: "success",
  missed_check_in: "warning",
  playing: "accent",
  eliminated: "neutral",
  withdrawn: "neutral",
  champion: "success",
};

/**
 * Um torneio de quem vê: nome (link), comunidade e início; a situação dele e,
 * se ainda joga, a próxima partida com o que ela espera.
 */
export function MyTournamentRow({ entry }: { entry: Schemas["MyTournament"] }) {
  const t = useTranslations("Home.tournaments");
  const tStatus = useTranslations("TournamentStatus");
  const locale = useLocale();
  const { tournament, placement } = entry;
  const path = `/${entry.tenant_slug}/${tournament.slug}`;
  const place = placement
    ? placement.tied > 1
      ? t("range", {
          from: placement.rank,
          to: placement.rank + placement.tied - 1,
        })
      : t("rank", { rank: placement.rank })
    : undefined;

  return (
    <li className="flex flex-col gap-1 py-1">
      <RowLink href={path} title={localizedName(tournament, locale)}>
        {entry.tenant_name} · {tStatus(tournament.status)}
        {tournament.starts_at && (
          <>
            {" · "}
            <LocalDateTime value={tournament.starts_at} />
          </>
        )}
      </RowLink>
      <div className="flex flex-col items-start gap-1 pb-2 sm:flex-row sm:flex-wrap sm:items-center sm:gap-3">
        <Badge variant={BADGE[entry.situation]}>
          {t(`situation.${entry.situation}`, { place: place ?? "" })}
        </Badge>
        {entry.next_match && (
          <NextMatchLink next={entry.next_match} tournamentPath={path} />
        )}
      </div>
    </li>
  );
}
