import { getTranslations } from "next-intl/server";

import { getTenant } from "@/features/tenants/server";
import { NewTournamentButton } from "./new-tournament-button";

/**
 * "Novo torneio", a ação primária do início da comunidade: só para a
 * organização e fora da exclusão agendada (a comunidade fica só leitura).
 * Fica em `<Suspense>`.
 */
export async function NewTournamentAction({ slug }: { slug: string }) {
  const [loaded, t] = await Promise.all([
    getTenant(slug),
    getTranslations("Tenant"),
  ]);
  if (loaded.kind !== "ok") return null;
  const { tenant } = loaded;
  if (tenant.role === "player" || tenant.deletion_scheduled_for) return null;

  return (
    <NewTournamentButton tenant={slug}>
      {t("newTournament")}
    </NewTournamentButton>
  );
}
