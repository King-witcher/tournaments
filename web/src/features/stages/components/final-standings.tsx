import { useTranslations } from "next-intl";

import { DepartureTag } from "@/features/matches/components/departure-tag";
import { memberPath } from "@/features/members/lib/paths";
import { useTeamName } from "@/features/teams/hooks/use-team-name";
import { soloMember, teamLeft } from "@/features/teams/lib/team-name";
import type { components } from "@/shared/api/schema";
import { Avatar } from "@/shared/ui/avatar";
import { DataTable } from "@/shared/ui/data-table";
import { Section } from "@/shared/ui/section";
import styles from "./final-standings.module.scss";

type Props = {
  tenant: string;
  /** Qualquer chave do torneio: leva a classificação final e os times dela. */
  bracket: components["schemas"]["TournamentBracket"];
};

/**
 * A classificação final do torneio em mais de uma fase, quando ele encerra:
 * posição (empatados dividem a faixa, 5º–8º) e o time (no duelo, o jogador,
 * com link para a pessoa), com quem desistiu ou saiu marcado. Os times vêm de
 * `entrants` e de `other_teams` (os que pararam numa fase anterior).
 */
export function FinalStandings({ tenant, bracket }: Props) {
  const t = useTranslations("Bracket.standings");
  const teamName = useTeamName();
  const byId = new Map(
    [...bracket.entrants, ...bracket.other_teams].map((team) => [
      team.id,
      team,
    ]),
  );

  const rows = bracket.standings_final.map((s) => {
    const team = byId.get(s.team_id);
    const name = team ? teamName(team) : "";
    const solo = team ? soloMember(team) : null;
    return {
      id: String(s.team_id),
      search: name,
      href: solo && !solo.left ? memberPath(tenant, solo.user_id) : undefined,
      cells: {
        rank:
          s.tied > 1
            ? t("range", { from: s.rank, to: s.rank + s.tied - 1 })
            : t("rank", { rank: s.rank }),
        player: (
          <span className={styles.player}>
            <Avatar src={solo?.avatar_url} name={name} />
            <span className={styles.name}>{name}</span>
            {team && (
              <DepartureTag withdrawn={team.withdrawn} left={teamLeft(team)} />
            )}
          </span>
        ),
      },
    };
  });

  return (
    <Section title={t("final")}>
      <DataTable
        label={t("final")}
        primaryColumn="player"
        columns={[
          { key: "rank", header: t("position") },
          { key: "player", header: t("player") },
        ]}
        rows={rows}
        empty={{ text: t("empty") }}
      />
    </Section>
  );
}
