import { Stack } from "@carbon/react";
import { getLocale, getTranslations } from "next-intl/server";

import { PendingMatches } from "@/features/matches/components/pending-matches";
import { loadBracket } from "@/features/matches/server";
import { Markdown } from "@/shared/ui/markdown";
import { Section } from "@/shared/ui/section";
import { localizedTexts } from "../lib/localize";
import { hasStarted } from "../lib/status";
import { loadTournament } from "../server";
import styles from "./tournament-overview.module.scss";
import { TournamentProgress } from "./tournament-progress";
import { TournamentSetupSummary } from "./tournament-setup-summary";

type Props = { tenant: string; slug: string };

/**
 * A aba Visão geral: o ciclo do torneio, as partidas que esperam uma ação de
 * quem vê (a partir do início), a descrição e o resumo do formato, lado a
 * lado a partir do `lg`. O erro e o login já aparecem na casca; aqui, nada.
 * Fica em `<Suspense>`.
 */
export async function TournamentOverview({ tenant, slug }: Props) {
  const [t, locale, loaded] = await Promise.all([
    getTranslations("Tournament"),
    getLocale(),
    loadTournament(tenant, slug),
  ]);
  if (loaded.kind !== "ok") return null;

  const { tournament } = loaded;
  // A chave só para as pendências; a mesma busca da aba da chave.
  const bracket = hasStarted(tournament.status)
    ? await loadBracket(tenant, slug)
    : undefined;
  const { description } = localizedTexts(tournament, locale);

  return (
    <Stack gap={7}>
      <TournamentProgress tournament={tournament} />
      {bracket?.kind === "ok" && (
        <PendingMatches
          bracket={bracket.data}
          basePath={`/${tenant}/${slug}/matches`}
        />
      )}
      <div className={description ? styles.columns : styles.single}>
        {description && (
          <Section title={t("description")}>
            <Markdown>{description}</Markdown>
          </Section>
        )}
        <TournamentSetupSummary tournament={tournament} />
      </div>
    </Stack>
  );
}
