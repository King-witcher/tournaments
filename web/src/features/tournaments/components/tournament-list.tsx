import { getTranslations } from "next-intl/server";

import { Link } from "@/i18n/navigation";
import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";
import { buttonVariants } from "@/shared/ui/button";
import { TournamentRow } from "./tournament-row";

type Props = {
  slug: string;
  /** Organizer e owner criam torneios e veem os rascunhos. */
  viewerRole: components["schemas"]["Role"];
};

/** Torneios do tenant. Fica em `<Suspense>`. */
export async function TournamentList({ slug, viewerRole }: Props) {
  const api = await serverApi();
  const [t, tournaments] = await Promise.all([
    getTranslations("Tournaments"),
    api
      .GET("/tenants/{slug}/tournaments", { params: { path: { slug } } })
      .then(({ data }) => data ?? []),
  ]);

  return (
    <section className="flex flex-col gap-3">
      <div className="flex items-center justify-between gap-4">
        <h2 className="text-xl font-semibold">{t("title")}</h2>
        {viewerRole !== "player" && (
          <Link
            href={`/${slug}/new`}
            className={buttonVariants({ variant: "outline", size: "sm" })}
          >
            {t("create")}
          </Link>
        )}
      </div>
      {tournaments.length === 0 ? (
        <p className="text-sm opacity-70">
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
