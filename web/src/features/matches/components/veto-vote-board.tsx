"use client";

import { useTranslations } from "next-intl";
import { useCallback } from "react";

import { useVetoBoard } from "../hooks/use-veto-board";
import { VetoTile } from "./veto-tile";
import styles from "./veto-vote-board.module.scss";

type Props = {
  /** A partida votada. */
  game: number;
};

/**
 * A votação do mapa de uma partida, nos mesmos cartões do veto: os mapas
 * oferecidos, quem votou em qual e, para quem ainda não votou, o cartão como
 * botão de votar. Some com menos de dois oferecidos (sem votação) e quando o
 * mapa sai.
 */
export function VetoVoteBoard({ game }: Props) {
  const t = useTranslations("Match.veto.vote");
  const { veto, names, pending, actions } = useVetoBoard();
  const { step } = actions;
  const vote = useCallback((map: string) => step({ map }), [step]);
  const g = veto?.games.find((x) => x.game === game);
  if (!veto || !g || g.map || g.offered.length < 2) return null;

  return (
    <div className={styles.board}>
      <p className={styles.hint}>{t("hint")}</p>
      <ul className={styles.maps}>
        {g.offered.map((m) => {
          const voters = (["a", "b"] as const)
            .filter((s) => g[`vote_${s}`] === m.slug)
            .map((s) => names[s]);
          const note =
            voters.length > 0
              ? t("votes", { names: voters.join(", ") })
              : undefined;
          return (
            <VetoTile
              key={m.slug}
              slug={m.slug}
              name={m.name}
              state="available"
              note={note}
              {...(veto.options.includes(m.slug) && {
                action: "pick" as const,
                actionText: t("vote"),
                actionLabel: t("voteFor", { map: m.name }),
                disabled: pending != null,
                onAction: vote,
              })}
            />
          );
        })}
      </ul>
    </div>
  );
}
