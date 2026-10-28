import { useTranslations } from "next-intl";

import { DepartureMark } from "@/features/participants/components/departure-mark";
import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import { useMatchLabels } from "../hooks/use-match-labels";

type Schemas = components["schemas"];

type Props = {
  side: Schemas["MatchSide"];
  entrants: Map<number, Schemas["BracketEntrant"]>;
  single: boolean;
  /** Placar deste lado, se há resultado. */
  score: string | null;
  winner: boolean;
  /** O placar ainda não vale (enviado, esperando confirmação). */
  tentative: boolean;
};

/** Um lado no cartão da partida: seed, apelido (ou de onde vem) e placar. */
export function MatchCardSide({
  side,
  entrants,
  single,
  score,
  winner,
  tentative,
}: Props) {
  const t = useTranslations("Bracket");
  const { source } = useMatchLabels(single);
  const entrant = side.user_id != null ? entrants.get(side.user_id) : undefined;
  const placeholder = side.bye
    ? t("bye")
    : side.from
      ? source(side.from)
      : t("tbd");

  return (
    <div className="flex items-center gap-2 px-3 py-1">
      <span className="w-5 text-right text-xs tabular-nums opacity-50">
        {entrant?.seed}
      </span>
      <span
        className={cn(
          "flex-1 truncate",
          entrant ? winner && "font-semibold" : "text-xs italic opacity-50",
        )}
      >
        {entrant?.nickname ?? placeholder}
      </span>
      {entrant && (
        <DepartureMark withdrawn={entrant.withdrawn} left={entrant.left} />
      )}
      {score != null && (
        <span
          className={cn(
            "tabular-nums",
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
