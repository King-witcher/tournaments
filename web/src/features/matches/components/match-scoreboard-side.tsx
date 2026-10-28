import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import { Avatar } from "@/shared/ui/avatar";
import { useMatchLabels } from "../hooks/use-match-labels";
import { DepartureTag } from "./departure-tag";
import styles from "./match-scoreboard-side.module.scss";
import { MemberLink } from "./member-link";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  entrant?: Schemas["BracketEntrant"] | null;
  side: Schemas["MatchSide"];
  single: boolean;
  /** Venceu a série. */
  winner: boolean;
  /** Lado direito: alinha ao fim. */
  alignEnd?: boolean;
};

/**
 * Um lado no placar da partida: avatar, apelido (link para a pessoa), seed e
 * a marca de quem saiu; sem jogador, de onde ele vem.
 */
export function MatchScoreboardSide({
  tenant,
  entrant,
  side,
  single,
  winner,
  alignEnd = false,
}: Props) {
  const t = useTranslations("Match");
  const { placeholder } = useMatchLabels(single);

  return (
    <div
      className={cn(
        styles.side,
        alignEnd && styles.end,
        winner && styles.winner,
      )}
    >
      {entrant ? (
        <>
          <Avatar src={entrant.avatar_url} name={entrant.nickname} size="lg" />
          <MemberLink
            tenant={tenant}
            userId={entrant.user_id}
            size="lg"
            className={styles.name}
          >
            {entrant.nickname}
          </MemberLink>
          <span className={styles.meta}>
            {t("seed", { seed: entrant.seed })}
            <DepartureTag withdrawn={entrant.withdrawn} left={entrant.left} />
          </span>
        </>
      ) : (
        <span className={styles.placeholder}>{placeholder(side)}</span>
      )}
    </div>
  );
}
