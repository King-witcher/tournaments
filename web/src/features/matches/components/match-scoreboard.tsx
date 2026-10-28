import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { MatchScoreboardSide } from "./match-scoreboard-side";

type Props = { detail: components["schemas"]["MatchDetail"] };

/**
 * O placar da série no topo: os dois lados com os mapas de cada um (os
 * confirmados, enquanto a série não fecha) e, embaixo, o estado: quantos
 * mapas vencem, quantos esperam confirmação ou estão contestados, ou como a
 * série foi decidida.
 */
export function MatchScoreboard({ detail }: Props) {
  const t = useTranslations("Match");
  const tBracket = useTranslations("Bracket");
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
  const needed = Math.floor(s.best_of / 2) + 1;

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

  return (
    <section className="flex flex-col gap-2">
      <div className="grid grid-cols-[1fr_auto_1fr] items-center gap-3 rounded-md border border-black/10 p-4 dark:border-white/15">
        <MatchScoreboardSide
          entrant={a}
          side={s.a}
          single={single}
          score={score("a")}
          winner={decided && r?.winner === "a"}
          tentative={!decided}
        />
        <span className="text-sm text-muted">{t("vs")}</span>
        <MatchScoreboardSide
          entrant={b}
          side={s.b}
          single={single}
          score={score("b")}
          winner={decided && r?.winner === "b"}
          tentative={!decided}
          alignEnd
        />
      </div>
      {lines.map((line, i) => (
        // biome-ignore lint/suspicious/noArrayIndexKey: linhas fixas, na mesma ordem.
        <p key={i} className="text-sm text-muted">
          {line}
        </p>
      ))}
    </section>
  );
}
