import { getTranslations } from "next-intl/server";

import { NextMatchLink } from "@/features/matches/components/next-match-link";
import { getMyTournamentEntry } from "@/features/tournaments/server";
import type { components } from "@/shared/api/schema";
import { StatusTag } from "@/shared/ui/status-tag";
import { SITUATION_STATUS } from "../lib/situation";
import { ParticipationTile } from "./participation-tile";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
};

/**
 * "Sua situação" depois do início, para o inscrito: na chave (com a próxima
 * partida como link), fora dela por não ter feito check-in, eliminado (com a
 * colocação), desistiu ou campeão. Lê "Seus torneios"; se a API falhar, cai
 * no que o torneio diz (fez ou não o check-in). Fica em `<Suspense>`.
 */
export async function MyBracketSituation({ tenant, tournament }: Props) {
  const [t, entry] = await Promise.all([
    getTranslations("Participants"),
    getMyTournamentEntry(tenant, tournament.slug),
  ]);
  const situation: components["schemas"]["Situation"] =
    entry?.situation ??
    (tournament.check_in_minutes != null && !tournament.checked_in
      ? "missed_check_in"
      : "playing");
  const placement = entry?.placement;
  const place = placement
    ? placement.tied > 1
      ? t("situation.range", {
          from: placement.rank,
          to: placement.rank + placement.tied - 1,
        })
      : t("situation.rank", { rank: placement.rank })
    : "";

  return (
    <ParticipationTile
      tag={
        <StatusTag status={SITUATION_STATUS[situation]}>
          {t(`situation.${situation}`, { place })}
        </StatusTag>
      }
    >
      {situation === "missed_check_in" && <p>{t("checkIn.missed")}</p>}
      {entry?.next_match && (
        <NextMatchLink
          next={entry.next_match}
          tournamentPath={`/${tenant}/${tournament.slug}`}
        />
      )}
    </ParticipationTile>
  );
}
