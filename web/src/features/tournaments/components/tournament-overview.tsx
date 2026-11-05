import { getLocale, getTranslations } from "next-intl/server";

import { PendingMatches } from "@/features/matches/components/pending-matches";
import { actionableMatches } from "@/features/matches/lib/status";
import { loadBracket } from "@/features/matches/server";
import { Stack } from "@/shared/ui/carbon/stack";
import { Section } from "@/shared/ui/section";
import { UserText } from "@/shared/ui/user-text";
import { localizedTexts } from "../lib/localize";
import { hasStarted } from "../lib/status";
import { loadTournament } from "../server";
import styles from "./tournament-overview.module.scss";
import { TournamentProgress } from "./tournament-progress";
import { TournamentSetupSummary } from "./tournament-setup-summary";

type Props = { tenant: string; slug: string };

/**
 * A aba Visão geral: o ciclo do torneio e, embaixo, a partir do `lg`, duas
 * colunas: à esquerda as séries que esperam uma ação de quem vê (a partir do
 * início) e a descrição; à direita o resumo do formato (com o link para a
 * aba Regras). O erro e o login já aparecem na casca; aqui, nada.
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
  const pending =
    bracket?.kind === "ok" && actionableMatches(bracket.data).length > 0
      ? bracket.data
      : undefined;
  const { description } = localizedTexts(tournament, locale);

  return (
    <Stack gap={7}>
      <TournamentProgress tournament={tournament} />
      <div className={pending || description ? styles.columns : styles.single}>
        {(pending || description) && (
          <Stack gap={7}>
            {pending && (
              <PendingMatches
                bracket={pending}
                basePath={`/${tenant}/${slug}/matches`}
              />
            )}
            {description && (
              <Section title={t("description")}>
                <UserText>{description}</UserText>
              </Section>
            )}
          </Stack>
        )}
        <TournamentSetupSummary
          tournament={tournament}
          rulesHref={`/${tenant}/${slug}/rules`}
        />
      </div>
    </Stack>
  );
}
