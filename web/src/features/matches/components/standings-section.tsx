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
};

/**
 * A classificação do torneio já iniciado: no round robin e no suíço, a tabela;
 * na eliminação, as colocações. Para a organização, em cima, os empates do
 * fim que esperam a decisão dela. Busca a chave, então fica em `<Suspense>`.
 */
export async function StandingsSection({
  tenant,
  tournament,
  organizer,
}: Props) {
  const loaded = await loadBracket(tenant, tournament.slug);
  if (loaded.kind === "error") {
    return <Notice kind="error">{await apiErrorMessage(loaded.code)}</Notice>;
  }
  const bracket = loaded.data;
  if (isTable(bracket.format)) {
    return (
      <Stack gap={7}>
        {organizer && (
          <PendingTies
            tenant={tenant}
            slug={tournament.slug}
            bracket={bracket}
          />
        )}
        <TableStandings tenant={tenant} bracket={bracket} />
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
