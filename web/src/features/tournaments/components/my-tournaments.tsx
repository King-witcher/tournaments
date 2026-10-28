import { getLocale, getTranslations } from "next-intl/server";

import { getMyTenants } from "@/features/tenants/server";
import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { RowLink } from "@/shared/ui/row-link";
import { checkInPending } from "../lib/check-in";
import { localizedName } from "../lib/localize";

type Schemas = components["schemas"];
type Entry = {
  tenant: Schemas["MyTenant"];
  tournament: Schemas["TournamentSummary"];
};

/** Em andamento primeiro: é onde há partida esperando. */
const ORDER: Schemas["TournamentStatus"][] = [
  "in_progress",
  "check_in",
  "registration",
];

/**
 * Na home, os torneios em que quem vê está inscrito e que ainda não
 * encerraram, de todas as comunidades: o caminho curto até o check-in e as
 * partidas. Sem nenhum, não mostra nada. Fica em `<Suspense>`.
 */
export async function MyTournaments() {
  const tenants = await getMyTenants();
  if (!tenants?.length) return null;

  const api = await serverApi();
  const lists = await Promise.all(
    tenants.map((tenant) =>
      api
        .GET("/tenants/{slug}/tournaments", {
          params: { path: { slug: tenant.slug } },
        })
        .then(({ data }) =>
          (data ?? []).map((tournament): Entry => ({ tenant, tournament })),
        )
        .catch((): Entry[] => []),
    ),
  );
  const entries = lists
    .flat()
    .filter(({ tournament: t }) => t.registered && ORDER.includes(t.status))
    .sort(
      (x, y) =>
        ORDER.indexOf(x.tournament.status) -
          ORDER.indexOf(y.tournament.status) ||
        (x.tournament.starts_at ?? "").localeCompare(
          y.tournament.starts_at ?? "",
        ),
    );
  if (entries.length === 0) return null;

  const [t, tList, tStatus, locale] = await Promise.all([
    getTranslations("Home.tournaments"),
    getTranslations("Tournaments"),
    getTranslations("TournamentStatus"),
    getLocale(),
  ]);
  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-xl font-semibold">{t("title")}</h2>
      <ul className="divide-y divide-black/10 dark:divide-white/10">
        {entries.map(({ tenant, tournament }) => (
          <li key={`${tenant.slug}/${tournament.slug}`}>
            <RowLink
              href={`/${tenant.slug}/${tournament.slug}`}
              title={localizedName(tournament, locale)}
            >
              {tenant.name} · {tStatus(tournament.status)}
              {tournament.starts_at && (
                <>
                  {" · "}
                  <LocalDateTime value={tournament.starts_at} />
                </>
              )}
              {checkInPending(tournament) && (
                <span className="font-medium text-red-700 dark:text-red-400">
                  {" · "}
                  {tList("checkInPending")}
                </span>
              )}
            </RowLink>
          </li>
        ))}
      </ul>
    </section>
  );
}
