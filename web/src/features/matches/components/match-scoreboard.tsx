import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { MatchScoreboardSide } from "./match-scoreboard-side";

type Props = { detail: components["schemas"]["MatchDetail"] };

/**
 * Os dois lados com o placar e, embaixo, o estado: a jogar, esperando
 * confirmação, contestada (com o comentário) ou como foi confirmada.
 */
export function MatchScoreboard({ detail }: Props) {
  const t = useTranslations("Match");
  const tBracket = useTranslations("Bracket");
  const { summary: s, a, b } = detail;
  const r = s.result;
  const single = detail.format.bracket === "single";
  const confirmed = s.state === "completed";
  const score = (side: "a" | "b") => {
    if (!r) return null;
    if (r.walkover) return r.winner === side ? tBracket("walkover") : "–";
    return String(side === "a" ? r.a_wins : r.b_wins);
  };

  let status: ReactNode = t(`state.${s.state}`);
  if (s.state === "ready" && r) {
    status = tBracket(
      `result.${r.status === "contested" ? "contested" : "pending"}`,
    );
  } else if (confirmed && r?.confirmed_via && r.confirmed_at) {
    const at = r.confirmed_at;
    status = t.rich(`confirmedVia.${r.confirmed_via}`, {
      time: () => <LocalDateTime value={at} />,
    });
  }

  return (
    <section className="flex flex-col gap-2">
      <div className="grid grid-cols-[1fr_auto_1fr] items-center gap-3 rounded-md border border-black/10 p-4 dark:border-white/15">
        <MatchScoreboardSide
          entrant={a}
          side={s.a}
          single={single}
          score={score("a")}
          winner={confirmed && r?.winner === "a"}
          tentative={!confirmed}
        />
        <span className="text-sm opacity-50">{t("vs")}</span>
        <MatchScoreboardSide
          entrant={b}
          side={s.b}
          single={single}
          score={score("b")}
          winner={confirmed && r?.winner === "b"}
          tentative={!confirmed}
          alignEnd
        />
      </div>
      <p className="text-sm opacity-70">{status}</p>
      {r?.status === "contested" && r.contest_comment && (
        <p className="text-sm">
          {t("waiting.contestComment", { comment: r.contest_comment })}
        </p>
      )}
    </section>
  );
}
