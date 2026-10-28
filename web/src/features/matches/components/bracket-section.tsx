import { getTranslations } from "next-intl/server";
import { apiErrorMessage } from "@/shared/api/errors";
import type { components } from "@/shared/api/schema";
import { Alert } from "@/shared/ui/alert";
import { loadBracket } from "../server";
import { BracketView } from "./bracket-view";
import { PendingMatches } from "./pending-matches";
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
      <PendingMatches
        bracket={bracket}
        basePath={`/${tenant}/${tournament.slug}/matches`}
      />
      {/*
        A chave sai da coluna do texto para a direita (até 72rem, sem passar
        da tela): com três ou mais rodadas ela não cabe na coluna, e a rolagem
        horizontal escondia a final.
      */}
      <section className="flex w-[min(100vw_-_2rem,50vw_+_19rem,72rem)] flex-col gap-3">
        <h2 className="text-xl font-semibold">{t("title")}</h2>
        {bracket.status === "in_progress" && (
          <p className="text-sm text-muted">{t("hint")}</p>
        )}
        {bracket.matches.some((m) => m.round > 1) && (
          // Na tela estreita só cabe uma rodada; as outras ficam na rolagem.
          <p className="text-sm text-muted sm:hidden">{t("scrollHint")}</p>
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
