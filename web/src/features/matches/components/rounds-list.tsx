import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Stack } from "@/shared/ui/carbon/stack";
import { MatchCard } from "./match-card";
import styles from "./rounds-list.module.scss";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  /** Slug do torneio. */
  tournament: string;
  bracket: Schemas["TournamentBracket"];
};

/**
 * As partidas do round robin e do suíço, por rodada ("Rodada 2 de 5"), com
 * os mesmos cartões da chave (código, lados, placar e o que a partida espera
 * de quem vê) e quem folga (round robin) ou recebe o bye (suíço). No round
 * robin, todas ficam abertas desde o início; no suíço, a rodada seguinte
 * aparece quando é pareada, e até lá uma linha diz quando isso acontece. Na
 * disputa de posições, a rodada única sem título: os pares, cada um com o
 * que está em jogo no lugar do código, e o sem par por último.
 */
export function RoundsList({ tenant, tournament, bracket }: Props) {
  const t = useTranslations("Bracket");
  const total = bracket.total_rounds ?? 0;
  const swiss = bracket.format.format === "swiss";
  const placement = bracket.format.format === "placement";
  const entrants = new Map(bracket.entrants.map((e) => [e.id, e]));
  const rounds = new Map<number, Schemas["BracketMatch"][]>();
  for (const m of bracket.matches) {
    rounds.set(m.round, [...(rounds.get(m.round) ?? []), m]);
  }
  const paired = Math.max(0, ...rounds.keys());
  const waiting = swiss && bracket.status === "in_progress" && paired < total;

  return (
    <Stack gap={7}>
      {[...rounds.entries()]
        .sort(([a], [b]) => a - b)
        .map(([round, matches]) => (
          <section key={round} className={styles.round}>
            {!placement && (
              <h3 className={styles.title}>{t("roundOf", { round, total })}</h3>
            )}
            <ol className={styles.matches}>
              {matches
                .sort((a, b) => a.index - b.index)
                .map((m) => (
                  <li key={m.key}>
                    <MatchCard
                      tenant={tenant}
                      match={m}
                      single={false}
                      entrants={entrants}
                      href={`/${tenant}/${tournament}/matches/${m.key}`}
                    />
                  </li>
                ))}
            </ol>
          </section>
        ))}
      {waiting && (
        <p className={styles.next}>
          {t("nextRound", { round: paired + 1, total, previous: paired })}
        </p>
      )}
    </Stack>
  );
}
