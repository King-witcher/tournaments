import type { components } from "@/shared/api/schema";
import { useMatchLabels } from "../hooks/use-match-labels";
import type { Round } from "../lib/bracket-layout";
import styles from "./bracket-part-grid.module.scss";
import { MatchCard } from "./match-card";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  /** Título da parte (só na dupla). */
  title?: string;
  part: Schemas["BracketPart"];
  rounds: Round[];
  single: boolean;
  entrants: Map<number, Schemas["BracketEntrant"]>;
  /** Endereço das partidas, sem a chave da partida. */
  basePath: string;
};

/**
 * Uma parte da chave: uma coluna por rodada, com as partidas distribuídas na
 * altura para lembrar o chaveamento. A rolagem para o lado é da chave
 * inteira (`BracketView`).
 */
export function BracketPartGrid({
  tenant,
  title,
  part,
  rounds,
  single,
  entrants,
  basePath,
}: Props) {
  const { roundTitle } = useMatchLabels(single);
  const lastRound = rounds[rounds.length - 1]?.round ?? 1;

  return (
    <div className={styles.part}>
      {title && <h3 className={styles.title}>{title}</h3>}
      <div className={styles.rounds}>
        {rounds.map(({ round, matches }) => (
          <div key={round} className={styles.round}>
            <h4 className={styles.roundTitle}>
              {roundTitle(part, round, lastRound)}
            </h4>
            <ol className={styles.matches}>
              {matches.map((m) => (
                <li key={m.key}>
                  <MatchCard
                    tenant={tenant}
                    match={m}
                    single={single}
                    entrants={entrants}
                    href={`${basePath}/${m.key}`}
                  />
                </li>
              ))}
            </ol>
          </div>
        ))}
      </div>
    </div>
  );
}
