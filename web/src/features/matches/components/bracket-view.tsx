import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { groupByRound } from "../lib/bracket-layout";
import { BracketPartGrid } from "./bracket-part-grid";
import styles from "./bracket-view.module.scss";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  tournament: string;
  bracket: Schemas["TournamentBracket"];
};

/**
 * Chave inteira: na simples, uma grade de rodadas; na dupla, a chave de
 * cima, a de baixo e a grande final, uma embaixo da outra. É o único lugar
 * do site com rolagem para o lado: um contêiner com rolagem própria, que
 * recebe o foco do teclado (as setas rolam) e tem nome para o leitor de
 * tela.
 */
export function BracketView({ tenant, tournament, bracket }: Props) {
  const t = useTranslations("Bracket");
  const single = bracket.format.bracket === "single";
  const entrants = new Map(bracket.entrants.map((e) => [e.user_id, e]));

  return (
    <section
      className={styles.scroller}
      aria-label={t("scrollLabel")}
      // biome-ignore lint/a11y/noNoninteractiveTabindex: a região que rola precisa do foco para rolar pelo teclado.
      tabIndex={0}
    >
      <div className={styles.parts}>
        {groupByRound(bracket.matches).map(({ part, rounds }) => (
          <BracketPartGrid
            key={part}
            tenant={tenant}
            title={single ? undefined : t(`parts.${part}`)}
            part={part}
            rounds={rounds}
            single={single}
            entrants={entrants}
            basePath={`/${tenant}/${tournament}/matches`}
          />
        ))}
      </div>
    </section>
  );
}
