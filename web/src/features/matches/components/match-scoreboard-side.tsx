import { useTranslations } from "next-intl";

import { TeamName } from "@/features/teams/components/team-name";
import { useTeamName } from "@/features/teams/hooks/use-team-name";
import { soloMember, teamLeft } from "@/features/teams/lib/team-name";
import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import { Avatar } from "@/shared/ui/avatar";
import { useMatchLabels } from "../hooks/use-match-labels";
import { DepartureTag } from "./departure-tag";
import styles from "./match-scoreboard-side.module.scss";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  team?: Schemas["Team"] | null;
  side: Schemas["MatchSide"];
  single: boolean;
  /** Venceu a série. */
  winner: boolean;
  /** Lado direito: alinha ao fim. */
  alignEnd?: boolean;
};

/**
 * Um lado no placar da partida: avatar, nome do time (no duelo, o apelido
 * como link para a pessoa), seed e a marca de quem saiu; sem time, de onde
 * ele vem.
 */
export function MatchScoreboardSide({
  tenant,
  team,
  side,
  single,
  winner,
  alignEnd = false,
}: Props) {
  const t = useTranslations("Match");
  const name = useTeamName();
  const { placeholder } = useMatchLabels(single);

  return (
    <div
      className={cn(
        styles.side,
        alignEnd && styles.end,
        winner && styles.winner,
      )}
    >
      {team ? (
        <>
          <Avatar
            src={soloMember(team)?.avatar_url}
            name={name(team)}
            size="lg"
          />
          <TeamName
            tenant={tenant}
            team={team}
            size="lg"
            className={styles.name}
          />
          <span className={styles.meta}>
            {t("seed", { seed: team.seed ?? "" })}
            <DepartureTag withdrawn={team.withdrawn} left={teamLeft(team)} />
          </span>
        </>
      ) : (
        <span className={styles.placeholder}>{placeholder(side)}</span>
      )}
    </div>
  );
}
