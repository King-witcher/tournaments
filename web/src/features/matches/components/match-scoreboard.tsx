import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import { Countdown } from "@/shared/ui/countdown";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { LocalTime } from "@/shared/ui/local-time";
import { useSideNames } from "../hooks/use-side-names";
import { SCORE_ID } from "../lib/match-target";
import styles from "./match-scoreboard.module.scss";
import { MatchScoreboardSide } from "./match-scoreboard-side";

type Props = {
  tenant: string;
  detail: components["schemas"]["MatchDetail"];
};

/**
 * O placar da série logo abaixo do cabeçalho: os dois lados com avatar e os
 * mapas de cada um em tipo grande (os confirmados, enquanto a série não
 * fecha); embaixo, o estado (quantos mapas vencem, quantos esperam
 * confirmação ou estão contestados, ou como a série foi decidida) e o prazo
 * do próximo mapa que confirma sozinho.
 */
export function MatchScoreboard({ tenant, detail }: Props) {
  const t = useTranslations("Match");
  const tBracket = useTranslations("Bracket");
  const names = useSideNames(detail);
  const { summary: s, a, b } = detail;
  const r = s.result;
  const single = detail.format.bracket === "single";
  const decided = s.state === "completed";
  const playing = s.state === "ready";
  const score = (side: "a" | "b") => {
    if (r?.walkover) return r.winner === side ? tBracket("walkover") : "–";
    if (r) return String(side === "a" ? r.a_wins : r.b_wins);
    if (playing) return String(side === "a" ? s.a_maps : s.b_maps);
    return null;
  };
  const scoreA = score("a");
  const scoreB = score("b");
  const needed = Math.floor(s.best_of / 2) + 1;
  // O mapa pendente que confirma primeiro (a linha dele tem o mesmo prazo).
  const due = detail.games
    .filter((g) => g.status === "pending" && g.confirm_by)
    .sort((x, y) => (x.confirm_by ?? "").localeCompare(y.confirm_by ?? ""))[0];

  const lines: ReactNode[] = [];
  if (decided && r) {
    const at = r.confirmed_at;
    lines.push(
      t.rich(`confirmedVia.${r.confirmed_via}`, {
        time: () => <LocalDateTime value={at} />,
      }),
    );
  } else if (playing) {
    lines.push(
      `${t("series.open")} ${t("series.needed", { bestOf: s.best_of, needed })}`,
    );
    if (s.pending_maps > 0)
      lines.push(t("series.pending", { count: s.pending_maps }));
    if (s.contested_maps > 0)
      lines.push(t("series.contested", { count: s.contested_maps }));
  } else {
    lines.push(t(`state.${s.state}`));
  }
  if (due?.confirm_by) {
    const until = due.confirm_by;
    lines.push(
      t.rich("score.deadline", {
        number: due.number,
        at: () => <LocalTime value={until} />,
        time: () => <Countdown until={until} />,
      }),
    );
  }

  return (
    // tabIndex -1: recebe o foco depois do W.O., que fecha o bloco da
    // organização com o botão usado.
    <section
      id={SCORE_ID}
      tabIndex={-1}
      aria-label={t("score.label")}
      className={styles.board}
    >
      <div className={styles.sides}>
        <MatchScoreboardSide
          tenant={tenant}
          team={a}
          side={s.a}
          single={single}
          winner={decided && r?.winner === "a"}
        />
        <p className={cn(styles.score, !decided && styles.tentative)}>
          {scoreA != null && scoreB != null && (
            <span className={styles.srOnly}>
              {t("score.sr", {
                a: names.a,
                b: names.b,
                aScore: scoreA,
                bScore: scoreB,
              })}
            </span>
          )}
          <span aria-hidden className={styles.digits}>
            <span
              className={cn(decided && r?.winner === "a" && styles.winnerScore)}
            >
              {scoreA}
            </span>
            <span className={styles.vs}>{t("vs")}</span>
            <span
              className={cn(decided && r?.winner === "b" && styles.winnerScore)}
            >
              {scoreB}
            </span>
          </span>
        </p>
        <MatchScoreboardSide
          tenant={tenant}
          team={b}
          side={s.b}
          single={single}
          winner={decided && r?.winner === "b"}
          alignEnd
        />
      </div>
      <div className={styles.lines}>
        {lines.map((line, i) => (
          // biome-ignore lint/suspicious/noArrayIndexKey: linhas fixas, na mesma ordem.
          <p key={i}>{line}</p>
        ))}
      </div>
    </section>
  );
}
