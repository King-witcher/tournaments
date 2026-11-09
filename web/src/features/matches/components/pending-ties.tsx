import { useLocale, useTranslations } from "next-intl";

import { useTeamName } from "@/features/teams/hooks/use-team-name";
import { soloMember } from "@/features/teams/lib/team-name";
import type { components } from "@/shared/api/schema";
import { Notice } from "@/shared/ui/notice";
import { type Tie, tieKey } from "../lib/ties";
import { PendingTiesAction } from "./pending-ties-action";
import {
  numberColumns,
  useRatingText,
  useStandingNumbers,
} from "./standing-numbers";

type Props = {
  tenant: string;
  slug: string;
  bracket: components["schemas"]["TournamentBracket"];
  /** Quem vê é organizer ou owner: decide os empates. */
  organizer: boolean;
  /** É a última fase: decididos os empates, o torneio termina. */
  lastStage: boolean;
  /** Veio de um "Decidir": a organização entra com o Modal aberto. */
  autoOpen?: boolean;
};

/**
 * Os empates do fim da fase que esperam a organização, em cima da tabela: para
 * ela, o aviso com a quantidade e o botão que abre a decisão (sempre montado,
 * para o Modal aberto sobreviver a um refresh sem nada pendente); para quem só
 * joga, quem empatou e que a organização vai decidir.
 */
export function PendingTies({
  tenant,
  slug,
  bracket,
  organizer,
  lastStage,
  autoOpen,
}: Props) {
  const t = useTranslations("Bracket.table");
  const locale = useLocale();
  const teamName = useTeamName();
  const numbers = useStandingNumbers(bracket.format.best_of);
  const ratingOf = useRatingText();
  const byId = new Map(bracket.entrants.map((e) => [e.id, e]));
  const rows = new Map((bracket.table ?? []).map((r) => [r.team_id, r]));
  const nameOf = (id: number) => {
    const team = byId.get(id);
    return team ? teamName(team) : "";
  };

  if (!organizer) {
    if (bracket.pending_ties.length === 0) return null;
    const names = new Intl.ListFormat(locale, { type: "conjunction" });
    const groups = bracket.pending_ties.map((tie) =>
      t("pendingGroup", {
        position: tie.position,
        names: names.format(tie.team_ids.map(nameOf)),
      }),
    );
    return (
      <Notice kind="info">{[...groups, t("pendingDecide")].join(" ")}</Notice>
    );
  }

  const columns = numberColumns(bracket.format);
  const labels = {
    record: t("record"),
    maps: t("maps"),
    frags: t("frags"),
    buchholz: t("buchholz"),
  };
  const ties: Tie[] = bracket.pending_ties.map((tie) => ({
    key: tieKey(tie.team_ids),
    position: tie.position,
    teams: tie.team_ids.map((id) => {
      const team = byId.get(id);
      const row = rows.get(id);
      const cells = row && numbers(row);
      return {
        id,
        name: nameOf(id),
        avatarUrl: team ? soloMember(team)?.avatar_url : null,
        numbers: cells
          ? [
              ...columns.map((column) => ({
                key: column,
                label: labels[column],
                value: cells[column],
              })),
              ...(row.rating != null
                ? [{ key: "rating", label: t("rating"), value: ratingOf(row) }]
                : []),
            ]
          : [],
      };
    }),
  }));

  return (
    <PendingTiesAction
      tenant={tenant}
      slug={slug}
      ties={ties}
      lastStage={lastStage}
      autoOpen={autoOpen}
    />
  );
}
