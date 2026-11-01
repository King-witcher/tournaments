import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import styles from "./game-frags.module.scss";

type Props = {
  game: components["schemas"]["GameDetail"];
  names: { a: string; b: string };
};

/**
 * Os frags de uma partida enviada, cada um embaixo do seu lado no cartão
 * (as colunas do alto), em tipo grande: o placar é o que se procura. O de
 * quem venceu (ou quem enviou diz que venceu) no texto principal, o outro em
 * cinza. Nada sem frags (partida antiga).
 */
export function GameFrags({ game: g, names }: Props) {
  const t = useTranslations("Match");
  if (g.a_frags == null || g.b_frags == null) return null;
  const value = (side: "a" | "b") => (
    <span className={cn(g.winner !== side && styles.loser)}>
      {side === "a" ? g.a_frags : g.b_frags}
    </span>
  );

  return (
    <p>
      <span className={styles.srOnly}>
        {t("score.sr", {
          a: names.a,
          b: names.b,
          aScore: g.a_frags,
          bScore: g.b_frags,
        })}
      </span>
      <span aria-hidden className={styles.row}>
        {value("a")}
        <span className={styles.vs}>{t("vs")}</span>
        {value("b")}
      </span>
    </p>
  );
}
