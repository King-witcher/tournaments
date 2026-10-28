import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import { Avatar } from "@/shared/ui/avatar";
import { useMatchLabels } from "../hooks/use-match-labels";
import { DepartureTag } from "./departure-tag";
import styles from "./match-card-side.module.scss";
import { MemberLink } from "./member-link";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  side: Schemas["MatchSide"];
  entrants: Map<number, Schemas["BracketEntrant"]>;
  single: boolean;
  /** Placar deste lado, se há resultado. */
  score: string | null;
  winner: boolean;
  /** O placar ainda não vale (a série não fechou). */
  tentative: boolean;
};

/**
 * Um lado no cartão da partida: seed, avatar e apelido (link para a pessoa,
 * por cima do link do cartão) ou de onde ele vem, a marca de quem saiu e o
 * placar. O avatar ajuda a achar alguém na chave grande antes de ler o
 * nome. `data-side` é onde chega a linha de quem vem da partida anterior.
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
  const entrant = side.user_id != null ? entrants.get(side.user_id) : undefined;

  return (
    <div className={cn(styles.side, winner && styles.winner)} data-side>
      <span className={styles.seed}>{entrant?.seed}</span>
      {entrant ? (
        <>
          <Avatar src={entrant.avatar_url} name={entrant.nickname} />
          <MemberLink
            tenant={tenant}
            userId={entrant.user_id}
            className={styles.name}
          >
            <span className={styles.nameText}>{entrant.nickname}</span>
          </MemberLink>
        </>
      ) : (
        <span className={styles.placeholder}>{placeholder(side)}</span>
      )}
      {entrant && (
        <DepartureTag withdrawn={entrant.withdrawn} left={entrant.left} />
      )}
      {score != null && (
        <span className={cn(styles.score, tentative && styles.tentative)}>
          {score}
        </span>
      )}
    </div>
  );
}
