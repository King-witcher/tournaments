import { getTranslations } from "next-intl/server";

import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";
import { ButtonLink } from "@/shared/ui/button-link";
import { TournamentRow } from "./tournament-row";

type Props = {
  slug: string;
  /** Organizer e owner criam torneios e veem os rascunhos. */
  viewerRole: components["schemas"]["Role"];
};

/**
 * Do que está acontecendo para o que já acabou; dentro de cada status, a
 * ordem da API (mais novo primeiro).
 */
const ORDER: components["schemas"]["TournamentStatus"][] = [
  "in_progress",
  "check_in",
  "registration",
  "draft",
  "finished",
];

/** Torneios do tenant. Fica em `<Suspense>`. */
export async function TournamentList({ slug, viewerRole }: Props) {
  const api = await serverApi();
  const [t, tournaments] = await Promise.all([
    getTranslations("Tournaments"),
    api
      .GET("/tenants/{slug}/tournaments", { params: { path: { slug } } })
      .then(({ data }) =>
        (data ?? []).toSorted(
          (a, b) => ORDER.indexOf(a.status) - ORDER.indexOf(b.status),
        ),
      ),
  ]);

  return (
    <section className="flex flex-col gap-3">
      <div className="flex items-center justify-between gap-4">
        <h2 className="text-xl font-semibold">{t("title")}</h2>
        {viewerRole !== "player" && (
          <ButtonLink href={`/${slug}/new`} kind="tertiary" size="sm">
            {t("create")}
          </ButtonLink>
        )}
      </div>
      {tournaments.length === 0 ? (
        <p className="text-sm text-muted">
          {viewerRole === "player" ? t("emptyPlayer") : t("empty")}
        </p>
      ) : (
        <ul className="divide-y divide-black/10 dark:divide-white/10">
          {tournaments.map((tournament) => (
            <TournamentRow
              key={tournament.slug}
              tenant={slug}
              tournament={tournament}
            />
          ))}
        </ul>
      )}
    </section>
  );
}
