import { Stack } from "@carbon/react";
import { getTranslations } from "next-intl/server";

import { isTable } from "@/features/tournaments/lib/format";
import { apiErrorMessage } from "@/shared/api/errors";
import type { components } from "@/shared/api/schema";
import { Notice } from "@/shared/ui/notice";
import { Section } from "@/shared/ui/section";
import { loadBracket } from "../server";
import styles from "./bracket-section.module.scss";
import { BracketView } from "./bracket-view";
import { RoundsList } from "./rounds-list";
import { StandingsTable } from "./standings-table";
import { TableStandings } from "./table-standings";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
};

/**
 * A chave do torneio em andamento ou encerrado, em largura cheia, e embaixo
 * a classificação (no round robin e no suíço, a tabela e as partidas por
 * rodada). Antes do início não mostra nada (a aba mostra o aviso).
 * Busca a chave, então fica em `<Suspense>`.
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
    return <Notice kind="error">{await apiErrorMessage(loaded.code)}</Notice>;
  }
  const bracket = loaded.data;
  // Round robin e suíço: a tabela em cima (é ela que classifica) e as
  // partidas por rodada embaixo, sem a grade da chave.
  if (isTable(bracket.format)) {
    return (
      <Stack gap={7}>
        <TableStandings tenant={tenant} bracket={bracket} />
        <RoundsList
          tenant={tenant}
          tournament={tournament.slug}
          bracket={bracket}
        />
      </Stack>
    );
  }
  const manyRounds = bracket.matches.some((m) => m.round > 1);

  return (
    <Stack gap={7}>
      <Section
        title={t("title")}
        description={
          (bracket.status === "in_progress" || manyRounds) && (
            <>
              {bracket.status === "in_progress" && t("hint")}
              {manyRounds && (
                // Na tela estreita só cabe uma rodada; as outras ficam na rolagem.
                <span className={styles.mobileOnly}> {t("scrollHint")}</span>
              )}
            </>
          )
        }
      >
        <BracketView
          tenant={tenant}
          tournament={tournament.slug}
          bracket={bracket}
        />
      </Section>
      <StandingsTable
        tenant={tenant}
        bracket={bracket}
        basePath={`/${tenant}/${tournament.slug}/matches`}
      />
    </Stack>
  );
}
