import { useTranslations } from "next-intl";
import { useTeamName } from "@/features/teams/hooks/use-team-name";
import type { components } from "@/shared/api/schema";
import { Link } from "@/shared/ui/link";
import { useNextMatchLabel } from "../hooks/use-next-match-label";
import { useTaskNoticeText } from "../hooks/use-task-notice-text";
import { isActionable, taskKey } from "../lib/status";

type Props = {
  next: components["schemas"]["NextMatch"];
  /** Endereço do torneio (`/{tenant}/{torneio}`). */
  tournamentPath: string;
};

/**
 * A próxima partida de quem vê, com o adversário (ou de onde ele vem), o que
 * ela espera dele (ou que o adversário está ocupado em outra) e, quando já
 * saiu, o próximo mapa, como link para a página da partida. O que pede uma
 * ação de quem vê vai em negrito, dito como o aviso da home ("P4 contestou
 * seu resultado", `useTaskNoticeText`); sem o detalhe, o imperativo de
 * `Bracket.task`.
 */
export function NextMatchLink({ next, tournamentPath }: Props) {
  const tBracket = useTranslations("Bracket");
  const label = useNextMatchLabel(next);
  const { title } = useTaskNoticeText();
  const teamName = useTeamName();
  const urgent = !next.opponent_playing && isActionable(next.task);
  const task = next.opponent_playing
    ? tBracket("opponentPlaying")
    : isActionable(next.task) && next.task_detail
      ? title(
          next.task,
          next.task_detail,
          next.opponent ? teamName(next.opponent) : undefined,
        )
      : next.task && tBracket(`task.${taskKey(next.task, next.veto)}`);

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
