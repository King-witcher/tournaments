import { TeamName } from "@/features/teams/components/team-name";
import { useTeamName } from "@/features/teams/hooks/use-team-name";
import { soloMember, teamLeft } from "@/features/teams/lib/team-name";
import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import { Avatar } from "@/shared/ui/avatar";
import { useMatchLabels } from "../hooks/use-match-labels";
import { DepartureTag } from "./departure-tag";
import styles from "./match-card-side.module.scss";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  side: Schemas["MatchSide"];
  /** Os times da chave, pelo id. */
  entrants: Map<number, Schemas["Team"]>;
  single: boolean;
  /** Placar deste lado, se há resultado. */
  score: string | null;
  winner: boolean;
  /** O placar ainda não vale (a série não fechou). */
  tentative: boolean;
};

/**
 * Um lado no cartão da partida: seed, avatar e nome do time (no duelo, o
 * apelido como link para a pessoa, por cima do link do cartão) ou de onde
 * ele vem, a marca de quem saiu e o placar. O avatar ajuda a achar alguém na
 * chave grande antes de ler o nome. `data-side` é onde chega a linha de quem
 * vem da partida anterior.
 */
export function MatchCardSide({
  tenant,
  side,
  entrants,
  single,
  score,
  winner,
  tentative,
}: Props) {
  const { placeholder } = useMatchLabels(single);
  const name = useTeamName();
  const team = side.team_id != null ? entrants.get(side.team_id) : undefined;

  return (
    <div className={cn(styles.side, winner && styles.winner)} data-side>
      <span className={styles.seed}>{team?.seed}</span>
      {team ? (
        <>
          <Avatar src={soloMember(team)?.avatar_url} name={name(team)} />
          <TeamName tenant={tenant} team={team} className={styles.name}>
            {(shown) => <span className={styles.nameText}>{shown}</span>}
          </TeamName>
        </>
      ) : (
        <span className={styles.placeholder}>{placeholder(side)}</span>
      )}
      {team && (
        <DepartureTag withdrawn={team.withdrawn} left={teamLeft(team)} />
      )}
      {score != null && (
        <span className={cn(styles.score, tentative && styles.tentative)}>
          {score}
        </span>
      )}
    </div>
  );
}
