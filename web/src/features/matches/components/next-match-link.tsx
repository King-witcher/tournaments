import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Link } from "@/shared/ui/link";
import { useNextMatchLabel } from "../hooks/use-next-match-label";
import { ACTIONABLE, taskKey } from "../lib/status";

type Props = {
  next: components["schemas"]["NextMatch"];
  /** Endereço do torneio (`/{tenant}/{torneio}`). */
  tournamentPath: string;
};

/**
 * A próxima partida de quem vê, com o adversário (ou de onde ele vem), o que
 * ela espera dele e, quando já saiu, o próximo mapa, como link para a página
 * da partida. O que pede uma ação de quem vê vai em negrito.
 */
export function NextMatchLink({ next, tournamentPath }: Props) {
  const tBracket = useTranslations("Bracket");
  const label = useNextMatchLabel(next);
  const task = next.task && tBracket(`task.${taskKey(next.task, next.veto)}`);
  const urgent =
    next.task !== undefined &&
    next.task !== null &&
    ACTIONABLE.includes(next.task);

  return (
    <Link href={`${tournamentPath}/matches/${next.key}`}>
      <span>
        {label}
        {task && (urgent ? <strong> · {task}</strong> : <> · {task}</>)}
        {next.next_map &&
          ` · ${tBracket("nextMap", { map: next.next_map.name })}`}
      </span>
    </Link>
  );
}
