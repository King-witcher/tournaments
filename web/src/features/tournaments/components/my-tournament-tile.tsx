import { useLocale, useTranslations } from "next-intl";

import { NextMatchLink } from "@/features/matches/components/next-match-link";
import type { components } from "@/shared/api/schema";
import { LinkTile } from "@/shared/ui/link-tile";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { type Status, StatusTag } from "@/shared/ui/status-tag";
import { localizedName } from "../lib/localize";

type Schemas = components["schemas"];

/** Cor da etiqueta pelo significado da situação de quem vê. */
const STATUS: Record<Schemas["Situation"], Status> = {
  registered: "registration",
  check_in_pending: "check_in",
  checked_in: "confirmed",
  missed_check_in: "cancelled",
  playing: "in_progress",
  eliminated: "finished",
  withdrawn: "cancelled",
  champion: "confirmed",
};

/**
 * Um torneio de quem vê, como tile: nome, comunidade e início, a situação
 * dele e, se ainda joga, a próxima partida (link à parte, embaixo).
 */
export function MyTournamentTile({
  entry,
}: {
  entry: Schemas["MyTournament"];
}) {
  const t = useTranslations("Home.tournaments");
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
    <LinkTile
      href={path}
      title={localizedName(tournament, locale)}
      meta={
        <>
          {entry.tenant_name}
          {tournament.starts_at && (
            <>
              {" · "}
              <LocalDateTime value={tournament.starts_at} />
            </>
          )}
        </>
      }
      tags={
        <StatusTag status={STATUS[entry.situation]}>
          {t(`situation.${entry.situation}`, { place: place ?? "" })}
        </StatusTag>
      }
      footer={
        entry.next_match && (
          <NextMatchLink next={entry.next_match} tournamentPath={path} />
        )
      }
    />
  );
}
