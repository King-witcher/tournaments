import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { useMatchLabels } from "../hooks/use-match-labels";
import { bracketColumns, bracketEdges } from "../lib/bracket-layout";
import styles from "./bracket-grid.module.scss";
import { BracketLines } from "./bracket-lines";
import { MatchCard } from "./match-card";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  /** As partes mostradas lado a lado (a principal e a grande final, ou a repescagem). */
  parts: Schemas["BracketPart"][];
  matches: Schemas["BracketMatch"][];
  single: boolean;
  entrants: Map<number, Schemas["BracketEntrant"]>;
  /** Endereço das partidas, sem a chave da partida. */
  basePath: string;
};

/**
 * Uma grade da chave: uma coluna por rodada, com as partidas distribuídas na
 * altura e as linhas que ligam cada uma às seguintes. É o único lugar do
 * site com rolagem para o lado: um contêiner com rolagem própria, que
 * recebe o foco do teclado (as setas rolam) e tem nome para o leitor de
 * tela.
 */
export function BracketGrid({
  tenant,
  parts,
  matches,
  single,
  entrants,
  basePath,
}: Props) {
  const t = useTranslations("Bracket");
  const { roundTitle } = useMatchLabels(single);
  const columns = bracketColumns(matches, parts);

  return (
    <section
      className={styles.scroller}
      aria-label={t("scrollLabel")}
      // biome-ignore lint/a11y/noNoninteractiveTabindex: a região que rola precisa do foco para rolar pelo teclado.
      tabIndex={0}
    >
      <BracketLines edges={bracketEdges(columns)}>
        <div className={styles.rounds}>
          {columns.map(({ part, round, lastRound, matches }) => (
            <div key={`${part}-${round}`} className={styles.round}>
              <h3 className={styles.roundTitle}>
                {roundTitle(part, round, lastRound)}
              </h3>
              <ol className={styles.matches}>
                {matches.map((m) => (
                  <li key={m.key} data-match={m.key}>
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
      </BracketLines>
    </section>
  );
}
