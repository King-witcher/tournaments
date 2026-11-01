import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { shownNickname } from "@/features/members/lib/nickname";
import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import styles from "./game-result.module.scss";

type Schemas = components["schemas"];

type Props = {
  game: Schemas["GameDetail"];
  names: { a: string; b: string };
  /** A célula da grade da linha. */
  className?: string;
};

/**
 * O resultado de uma partida enviada: quem venceu (sem confirmar, como o que
 * quem enviou diz), os frags e, no TDM, quem jogou por lado.
 */
export function GameResult({ game: g, names, className }: Props) {
  const t = useTranslations("Match.games");
  const tMember = useTranslations("Member");
  const winner = names[g.winner];
  const settled = g.status === "confirmed";
  const hidden = (chunks: ReactNode) => (
    <span className={styles.srOnly}>{chunks}</span>
  );
  // TDM: quem jogou por lado (no duelo, o próprio jogador: não aparece).
  const nicks = (list: Schemas["TeamMember"][]) =>
    list
      .map((m) => shownNickname(m.nickname, tMember("deletedAccount")))
      .join(", ");
  const lineup =
    g.lineup_a.length > 1 || g.lineup_b.length > 1
      ? t("lineup", { a: nicks(g.lineup_a), b: nicks(g.lineup_b) })
      : null;

  return (
    // Negrito só no que vale: sem confirmar, é a palavra de quem enviou, e o
    // placar no topo ainda não conta a partida.
    <span className={cn(styles.result, settled && styles.settled, className)}>
      {settled || !g.reported_side
        ? t("won", { name: winner })
        : t("claim", {
            reporter: names[g.reported_side],
            winner,
            // "Fulano enviou que venceu", sem o nome duas vezes.
            self: g.reported_side === g.winner ? "yes" : "no",
          })}
      {g.a_frags != null && g.b_frags != null && (
        <span className={styles.frags}>
          {t.rich("fragsCell", { a: g.a_frags, b: g.b_frags, prefix: hidden })}
        </span>
      )}
      {lineup && <span className={styles.lineup}>{lineup}</span>}
    </span>
  );
}
