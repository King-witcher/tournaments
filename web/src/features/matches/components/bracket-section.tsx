import { Stack } from "@carbon/react";
import { getTranslations } from "next-intl/server";

import { apiErrorMessage } from "@/shared/api/errors";
import type { components } from "@/shared/api/schema";
import { Notice } from "@/shared/ui/notice";
import { Section } from "@/shared/ui/section";
import { loadBracket } from "../server";
import styles from "./bracket-section.module.scss";
import { BracketView } from "./bracket-view";
import { StandingsTable } from "./standings-table";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
};

/**
 * A chave do torneio em andamento ou encerrado, em largura cheia, e embaixo
 * a classificação. Antes do início não mostra nada (a aba mostra o aviso).
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
