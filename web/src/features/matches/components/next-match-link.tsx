import { useTranslations } from "next-intl";

import { Link } from "@/i18n/navigation";
import type { components } from "@/shared/api/schema";
import { useMatchLabels } from "../hooks/use-match-labels";

type Props = {
  next: components["schemas"]["NextMatch"];
  /** Endereço do torneio (`/{tenant}/{torneio}`). */
  tournamentPath: string;
};

/** O que pede uma ação de quem vê: vai em negrito. */
const ACTIONABLE: components["schemas"]["ViewerTask"][] = [
  "report",
  "respond",
  "resolve",
];

/**
 * A próxima partida de quem vê, com o adversário (ou de onde ele vem) e o que
 * ela espera dele, como link para a página da partida.
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

  return (
    <Link
      href={`${tournamentPath}/matches/${next.key}`}
      className="inline-flex min-h-11 items-center self-start text-sm text-link underline underline-offset-4 sm:min-h-0"
    >
      <span>
        {label}
        {next.task && (
          <span
            className={
              ACTIONABLE.includes(next.task) ? "font-semibold" : undefined
            }
          >
            {" · "}
            {tTask(next.task)}
          </span>
        )}
      </span>
    </Link>
  );
}
