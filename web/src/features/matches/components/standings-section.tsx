import { FinalStandings } from "@/features/stages/components/final-standings";
import { isLastStage, withStageStatus } from "@/features/stages/lib/stages";
import { isTable } from "@/features/tournaments/lib/format";
import { apiErrorMessage } from "@/shared/api/errors";
import type { components } from "@/shared/api/schema";
import { Stack } from "@/shared/ui/carbon/stack";
import { Notice } from "@/shared/ui/notice";
import { loadBracket } from "../server";
import { PendingTies } from "./pending-ties";
import { StandingsTable } from "./standings-table";
import { TableStandings } from "./table-standings";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
  /** Quem vê é organizer ou owner: decide os empates do fim. */
  organizer: boolean;
  /** A fase pedida; sem ela, a atual. */
  stage?: number;
  /** A classificação final do torneio em mais de uma fase, no lugar da de uma fase. */
  final?: boolean;
  /** Veio de um "Decidir": a organização entra com o Modal dos empates aberto. */
  openTies?: boolean;
};

/**
 * A classificação do torneio já iniciado: no round robin e no suíço, a tabela;
 * na eliminação, as colocações. Em cima da tabela, o aviso dos empates do fim
 * que esperam a organização (ela decide por ele). Busca a chave, então fica em
 * `<Suspense>`.
 */
export async function StandingsSection({
  tenant,
  tournament,
  organizer,
  stage,
  final,
  openTies,
}: Props) {
  const loaded = await loadBracket(tenant, tournament.slug, stage);
  if (loaded.kind === "error") {
    return <Notice kind="error">{await apiErrorMessage(loaded.code)}</Notice>;
  }
  // A fase que já terminou tem a classificação definitiva, mesmo com o
  // torneio em andamento; o 1º só venceu o torneio na última fase.
  const bracket = withStageStatus(tournament, loaded.data);
  if (final) return <FinalStandings tenant={tenant} bracket={bracket} />;
  if (isTable(bracket.format)) {
    const lastStage = isLastStage(tournament, bracket.stage);
    return (
      <Stack gap={5}>
        <PendingTies
          tenant={tenant}
          slug={tournament.slug}
          bracket={bracket}
          organizer={organizer}
          lastStage={lastStage}
          autoOpen={openTies}
        />
        <TableStandings
          tenant={tenant}
          bracket={bracket}
          lastStage={lastStage}
        />
      </Stack>
    );
  }
  return (
    <StandingsTable
      tenant={tenant}
      bracket={bracket}
      basePath={`/${tenant}/${tournament.slug}/matches`}
    />
  );
}
