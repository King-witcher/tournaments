import { getTranslations } from "next-intl/server";

import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";
import { SeedingEditor } from "./seeding-editor";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
};

/**
 * Para organizer e owner, na revisão antes de iniciar: os elegíveis na ordem
 * que vai para a chave, com sorteio e ajuste à mão. Só no check-in, ou na
 * inscrição de torneio sem check-in. Fica em `<Suspense>`.
 */
export async function SeedingPanel({ tenant, tournament }: Props) {
  const hasCheckIn = tournament.check_in_minutes != null;
  const reviewing =
    tournament.status === "check_in" ||
    (tournament.status === "registration" && !hasCheckIn);
  if (!reviewing) return null;

  const api = await serverApi();
  const [t, order] = await Promise.all([
    getTranslations("Participants.seeding"),
    api
      .GET("/tenants/{slug}/tournaments/{tournament}/seeding", {
        params: { path: { slug: tenant, tournament: tournament.slug } },
      })
      .then(({ data }) => data ?? []),
  ]);

  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-xl font-semibold">
        {t("title", { count: order.length, total: tournament.participants })}
      </h2>
      <p className="text-sm text-muted">{t("hint")}</p>
      {order.length === 0 ? (
        <p className="text-sm text-muted">
          {hasCheckIn ? t("empty") : t("emptyNoCheckIn")}
        </p>
      ) : (
        <SeedingEditor
          // Recomeça do que a API devolveu a cada atualização da página.
          key={order.map((p) => `${p.user_id}:${p.seed}`).join(",")}
          tenant={tenant}
          tournament={tournament.slug}
          initial={order}
        />
      )}
    </section>
  );
}
