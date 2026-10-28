import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Link } from "@/shared/ui/link";
import { useMatchLabels } from "../hooks/use-match-labels";
import { ACTIONABLE } from "../lib/status";

type Props = {
  next: components["schemas"]["NextMatch"];
  /** Endereço do torneio (`/{tenant}/{torneio}`). */
  tournamentPath: string;
};

/**
 * A próxima partida de quem vê, com o adversário (ou de onde ele vem) e o que
 * ela espera dele, como link para a página da partida. O que pede uma ação
 * de quem vê vai em negrito.
 */
export function NextMatchLink({ next, tournamentPath }: Props) {
  const t = useTranslations("Home.tournaments.next");
  const tTask = useTranslations("Bracket.task");
  const { code } = useMatchLabels(next.single);
  const match = code(next.key);
  const from = next.opponent_from;
  const label = next.opponent
    ? t("against", { match, opponent: next.opponent })
    : from
      ? t(from.outcome, { match, source: code(from.match_key) })
      : t("unknown", { match });
  const task = next.task && tTask(next.task);
  const urgent =
    next.task !== undefined &&
    next.task !== null &&
    ACTIONABLE.includes(next.task);

  return (
    <Link href={`${tournamentPath}/matches/${next.key}`}>
      <span>
        {label}
        {task && (urgent ? <strong> · {task}</strong> : <> · {task}</>)}
      </span>
    </Link>
  );
}
