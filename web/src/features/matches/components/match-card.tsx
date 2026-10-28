import { useTranslations } from "next-intl";

import { Link } from "@/i18n/navigation";
import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import { useMatchLabels } from "../hooks/use-match-labels";
import { MatchCardSide } from "./match-card-side";
import { MatchStatusBadge } from "./match-status-badge";

type Schemas = components["schemas"];

type Props = {
  match: Schemas["BracketMatch"];
  single: boolean;
  entrants: Map<number, Schemas["BracketEntrant"]>;
  href: string;
};

/**
 * Os placares de cada lado no cartão: o da série decidida ou, enquanto ela
 * corre, os mapas confirmados de cada um.
 */
function scores(
  m: Schemas["BracketMatch"],
  walkover: string,
): [string | null, string | null] {
  const r = m.result;
  if (r?.walkover) return r.winner === "a" ? [walkover, "–"] : ["–", walkover];
  if (r) return [String(r.a_wins), String(r.b_wins)];
  const played = m.a_maps + m.b_maps + m.pending_maps + m.contested_maps > 0;
  if (m.state === "ready" && played)
    return [String(m.a_maps), String(m.b_maps)];
  return [null, null];
}

/**
 * Uma partida na chave: os dois lados (seed, apelido, placar), o estado e o
 * que ela espera de quem vê. As de quem vê ficam destacadas, e mais ainda as
 * que pedem ação. Bye e reset que não precisou ficam discretos.
 */
export function MatchCard({ match: m, single, entrants, href }: Props) {
  const t = useTranslations("Bracket");
  const { code } = useMatchLabels(single);

  if (m.state === "bye" || m.state === "not_needed") {
    const who = entrants.get(m.a.user_id ?? m.b.user_id ?? -1);
    return (
      <div className="rounded-md border border-dashed border-black/15 px-3 py-2 text-xs text-muted dark:border-white/20">
        {m.state === "not_needed"
          ? `${code(m.key)} · ${t("notNeeded")}`
          : who
            ? `${who.nickname} ${t("advances")}`
            : t("bye")}
      </div>
    );
  }

  const [scoreA, scoreB] = scores(m, t("walkover"));
  const confirmed = m.state === "completed";
  const winner = confirmed ? m.result?.winner : undefined;
  const actionable =
    m.task === "report" || m.task === "respond" || m.task === "resolve";
  return (
    <Link
      href={href}
      className={cn(
        "block rounded-md border text-sm hover:bg-black/5 dark:hover:bg-white/5",
        m.yours ? "border-blue-500/70" : "border-black/15 dark:border-white/20",
        actionable && "ring-2 ring-blue-500/40",
      )}
    >
      <div className="flex justify-between px-3 pt-1.5 text-xs text-muted">
        <span>{code(m.key)}</span>
        <span>{t("bestOf", { count: m.best_of })}</span>
      </div>
      <MatchCardSide
        side={m.a}
        entrants={entrants}
        single={single}
        score={scoreA}
        winner={winner === "a"}
        tentative={!confirmed}
      />
      <MatchCardSide
        side={m.b}
        entrants={entrants}
        single={single}
        score={scoreB}
        winner={winner === "b"}
        tentative={!confirmed}
      />
      <MatchStatusBadge match={m} />
    </Link>
  );
}
