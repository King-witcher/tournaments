import { useTranslations } from "next-intl";

import { DepartureMark } from "@/features/participants/components/departure-mark";
import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import { Avatar } from "@/shared/ui/avatar";
import { useMatchLabels } from "../hooks/use-match-labels";

type Schemas = components["schemas"];

type Props = {
  entrant?: Schemas["BracketEntrant"] | null;
  side: Schemas["MatchSide"];
  single: boolean;
  score: string | null;
  winner: boolean;
  /** O placar ainda não vale. */
  tentative: boolean;
  /** Lado direito: alinha ao fim. */
  alignEnd?: boolean;
};

/** Um lado no placar da página da partida. */
export function MatchScoreboardSide({
  entrant,
  side,
  single,
  score,
  winner,
  tentative,
  alignEnd = false,
}: Props) {
  const t = useTranslations("Match");
  const tBracket = useTranslations("Bracket");
  const { source } = useMatchLabels(single);
  const placeholder = side.bye
    ? tBracket("bye")
    : side.from
      ? source(side.from)
      : tBracket("tbd");

  return (
    <div
      className={cn(
        "flex items-center gap-3",
        alignEnd && "flex-row-reverse text-right",
      )}
    >
      {entrant && <Avatar src={entrant.avatar_url} />}
      <div className="flex min-w-0 flex-1 flex-col">
        <span
          className={cn(
            "truncate",
            entrant ? winner && "font-semibold" : "text-sm italic opacity-60",
          )}
        >
          {entrant?.nickname ?? placeholder}
        </span>
        {entrant && (
          <span
            className={cn(
              "flex gap-2 text-xs opacity-60",
              alignEnd && "justify-end",
            )}
          >
            {t("seed", { seed: entrant.seed })}
            <DepartureMark withdrawn={entrant.withdrawn} left={entrant.left} />
          </span>
        )}
      </div>
      {score != null && (
        <span
          className={cn(
            "text-2xl tabular-nums",
            winner && "font-semibold",
            tentative && "italic opacity-60",
          )}
        >
          {score}
        </span>
      )}
    </div>
  );
}
