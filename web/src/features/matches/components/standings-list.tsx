import { useTranslations } from "next-intl";

import { DepartureMark } from "@/features/participants/components/departure-mark";
import type { components } from "@/shared/api/schema";
import { Avatar } from "@/shared/ui/avatar";

type Schemas = components["schemas"];

type Props = {
  standings: Schemas["Standing"][];
  entrants: Schemas["BracketEntrant"][];
};

/** Classificação final: empatados dividem a faixa (5º–8º). */
export function StandingsList({ standings, entrants }: Props) {
  const t = useTranslations("Bracket.standings");
  const byUser = new Map(entrants.map((e) => [e.user_id, e]));

  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-xl font-semibold">{t("title")}</h2>
      <ol className="divide-y divide-black/10 dark:divide-white/10">
        {standings.map((s) => {
          const entrant = byUser.get(s.user_id);
          return (
            <li key={s.user_id} className="flex items-center gap-3 py-2">
              <span className="w-14 text-right text-sm font-medium tabular-nums">
                {s.tied > 1
                  ? t("range", { from: s.rank, to: s.rank + s.tied - 1 })
                  : t("rank", { rank: s.rank })}
              </span>
              <Avatar src={entrant?.avatar_url} />
              <span className={s.rank === 1 ? "font-semibold" : "font-medium"}>
                {entrant?.nickname}
              </span>
              {entrant && (
                <DepartureMark
                  withdrawn={entrant.withdrawn}
                  left={entrant.left}
                />
              )}
            </li>
          );
        })}
      </ol>
    </section>
  );
}
