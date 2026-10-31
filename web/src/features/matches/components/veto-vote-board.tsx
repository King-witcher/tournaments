"use client";

import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Button } from "@/shared/ui/button";
import { ErrorText } from "@/shared/ui/error-text";
import { useVetoAction } from "../hooks/use-veto-action";
import type { MatchTarget } from "../lib/match-target";
import styles from "./veto-board.module.scss";
import { VetoMapTile } from "./veto-map-tile";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  veto: Schemas["VetoView"];
  game: Schemas["VetoGame"];
  names: { a: string; b: string };
};

/**
 * A votação do mapa de um jogo: os mapas oferecidos, em cartões com a
 * imagem, quem votou em qual e, para quem ainda não votou, o botão de votar. Votos iguais, é esse;
 * diferentes, sorteio entre os dois.
 */
export function VetoVoteBoard({ target, veto, game, names }: Props) {
  const t = useTranslations("Match.veto.vote");
  const action = useVetoAction(target);
  if (game.offered.length < 2) return null;

  return (
    <div className={styles.board}>
      <h3 className={styles.heading}>{t("title", { game: game.game })}</h3>
      <p className={styles.reason}>{t("hint")}</p>
      <ul className={styles.maps}>
        {game.offered.map((m) => {
          const voters = (["a", "b"] as const)
            .filter((s) => game[`vote_${s}`] === m.slug)
            .map((s) => names[s]);
          return (
            <VetoMapTile
              key={m.slug}
              map={m}
              state={
                voters.length > 0
                  ? t("votes", { names: voters.join(", ") })
                  : t("noVotes")
              }
            >
              {veto.options.includes(m.slug) && (
                <Button
                  size="sm"
                  pending={action.pending}
                  onClick={() => action.step({ map: m.slug })}
                  aria-label={t("voteFor", { map: m.name })}
                >
                  {t("vote")}
                </Button>
              )}
            </VetoMapTile>
          );
        })}
      </ul>
      {game.map && (
        <p className={styles.summary}>{t("result", { map: game.map.name })}</p>
      )}
      {action.error && <ErrorText>{action.error}</ErrorText>}
    </div>
  );
}
