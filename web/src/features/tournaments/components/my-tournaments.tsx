import { getTranslations } from "next-intl/server";

import { serverApi } from "@/shared/api/server";
import { MyTournamentRow } from "./my-tournament-row";

/**
 * Na home, os torneios em que quem vê está inscrito, de todas as
 * comunidades, com a situação dele e a próxima partida: o caminho curto até
 * o check-in e o envio dos mapas. Os encerrados recentes vêm depois, numa
 * lista à parte. Sem nenhum (ou sem login), não mostra nada. Fica em
 * `<Suspense>`.
 */
export async function MyTournaments() {
  const api = await serverApi();
  const entries = await api
    .GET("/me/tournaments")
    .then(({ data }) => data ?? [])
    .catch(() => []);
  if (entries.length === 0) return null;

  const active = entries.filter((e) => e.tournament.status !== "finished");
  const finished = entries.filter((e) => e.tournament.status === "finished");
  const t = await getTranslations("Home.tournaments");
  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-xl font-semibold">{t("title")}</h2>
      {active.length === 0 ? (
        <p className="text-sm text-muted">{t("noneActive")}</p>
      ) : (
        <ul className="divide-y divide-black/10 dark:divide-white/10">
          {active.map((entry) => (
            <MyTournamentRow
              key={`${entry.tenant_slug}/${entry.tournament.slug}`}
              entry={entry}
            />
          ))}
        </ul>
      )}
      {finished.length > 0 && (
        <div className="flex flex-col gap-1">
          <h3 className="font-semibold">{t("finished")}</h3>
          <ul className="divide-y divide-black/10 dark:divide-white/10">
            {finished.map((entry) => (
              <MyTournamentRow
                key={`${entry.tenant_slug}/${entry.tournament.slug}`}
                entry={entry}
              />
            ))}
          </ul>
        </div>
      )}
    </section>
  );
}
