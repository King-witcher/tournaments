import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { MatchEventRow } from "./match-event-row";

type Props = { detail: components["schemas"]["MatchDetail"] };

/** Tudo o que aconteceu na partida, com quem e quando, do mais antigo. */
export function MatchHistory({ detail }: Props) {
  const t = useTranslations("Match.history");

  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-xl font-semibold">{t("title")}</h2>
      {detail.history.length === 0 ? (
        <p className="text-sm text-muted">{t("empty")}</p>
      ) : (
        <ol className="flex flex-col gap-3 border-l border-black/10 pl-4 dark:border-white/15">
          {detail.history.map((event) => (
            <MatchEventRow
              key={`${event.at}:${event.kind}:${event.game}`}
              event={event}
            />
          ))}
        </ol>
      )}
    </section>
  );
}
