import { isTable } from "@/features/tournaments/lib/format";
import { apiErrorMessage } from "@/shared/api/errors";
import type { components } from "@/shared/api/schema";
import { Notice } from "@/shared/ui/notice";
import { loadBracket } from "../server";
import { StandingsTable } from "./standings-table";
import { TableStandings } from "./table-standings";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
};

/**
 * A classificação do torneio já iniciado: no round robin e no suíço, a tabela;
 * na eliminação, as colocações. Busca a chave, então fica em `<Suspense>`.
 */
export async function StandingsSection({ tenant, tournament }: Props) {
  const loaded = await loadBracket(tenant, tournament.slug);
  if (loaded.kind === "error") {
    return <Notice kind="error">{await apiErrorMessage(loaded.code)}</Notice>;
  }
  const bracket = loaded.data;
  if (isTable(bracket.format)) {
    return <TableStandings tenant={tenant} bracket={bracket} />;
  }
  return (
    <StandingsTable
      tenant={tenant}
      bracket={bracket}
      basePath={`/${tenant}/${tournament.slug}/matches`}
    />
  );
}
