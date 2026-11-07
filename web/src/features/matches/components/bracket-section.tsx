import { getTranslations } from "next-intl/server";

import { withStageStatus } from "@/features/stages/lib/stages";
import { isTable } from "@/features/tournaments/lib/format";
import { apiErrorMessage } from "@/shared/api/errors";
import type { components } from "@/shared/api/schema";
import { Notice } from "@/shared/ui/notice";
import { Section } from "@/shared/ui/section";
import { loadBracket } from "../server";
import styles from "./bracket-section.module.scss";
import { BracketView } from "./bracket-view";
import { PlacementMatches } from "./placement-matches";
import { TableMatches } from "./table-matches";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
  /** A fase pedida; sem ela, a atual. */
  stage?: number;
};

/**
 * A chave do torneio em andamento ou encerrado, em largura cheia (no round
 * robin, no suíço e na disputa de posições, as séries). A classificação tem
 * a aba dela. Antes do início não mostra nada (a aba mostra o aviso).
 * Busca a chave, então fica em `<Suspense>`.
 */
export async function BracketSection({ tenant, tournament, stage }: Props) {
  if (tournament.status !== "in_progress" && tournament.status !== "finished") {
    return null;
  }
  const [t, loaded] = await Promise.all([
    getTranslations("Bracket"),
    loadBracket(tenant, tournament.slug, stage),
  ]);
  if (loaded.kind === "error") {
    return <Notice kind="error">{await apiErrorMessage(loaded.code)}</Notice>;
  }
  // A fase que já terminou sai encerrada, sem as dicas de quem ainda joga.
  const bracket = withStageStatus(tournament, loaded.data);
  // Round robin e suíço: as partidas por rodada, sem a grade da chave.
  if (isTable(bracket.format)) {
    return (
      <TableMatches
        tenant={tenant}
        tournament={tournament.slug}
        bracket={bracket}
      />
    );
  }
  // Disputa de posições: os pares, sem linhas entre eles.
  if (bracket.format.format === "placement") {
    return (
      <PlacementMatches
        tenant={tenant}
        tournament={tournament.slug}
        bracket={bracket}
      />
    );
  }
  const manyRounds = bracket.matches.some((m) => m.round > 1);

  return (
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
  );
}
