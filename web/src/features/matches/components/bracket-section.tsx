import { getTranslations } from "next-intl/server";
import { apiErrorMessage } from "@/shared/api/errors";
import type { components } from "@/shared/api/schema";
import { Alert } from "@/shared/ui/alert";
import { loadBracket } from "../server";
import { BracketView } from "./bracket-view";
import { StandingsList } from "./standings-list";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
};

/**
 * A chave do torneio em andamento ou encerrado e, quando encerra, a
 * classificação. Busca a chave, então fica em `<Suspense>`.
 */
export async function BracketSection({ tenant, tournament }: Props) {
  if (tournament.status !== "in_progress" && tournament.status !== "finished") {
    return null;
  }
  const [t, loaded] = await Promise.all([
    getTranslations("Bracket"),
    loadBracket(tenant, tournament.slug),
  ]);
  if (loaded.kind === "error") {
    return <Alert>{await apiErrorMessage(loaded.code)}</Alert>;
  }
  const bracket = loaded.data;

  return (
    <>
      {bracket.status === "finished" && (
        <StandingsList
          standings={bracket.standings}
          entrants={bracket.entrants}
        />
      )}
      <section className="flex flex-col gap-3">
        <h2 className="text-xl font-semibold">{t("title")}</h2>
        {bracket.status === "in_progress" && (
          <p className="text-sm opacity-70">{t("hint")}</p>
        )}
        <BracketView
          tenant={tenant}
          tournament={tournament.slug}
          bracket={bracket}
        />
      </section>
    </>
  );
}
