import { Stack } from "@/shared/ui/carbon/stack";
import { SectionSkeleton, TableSkeleton } from "@/shared/ui/skeleton";

/**
 * A lista de torneios do início da comunidade carregando, na forma dela: os
 * ativos em tiles e a tabela dos encerrados (`TournamentList`). Só visual: o
 * "Carregando…" do leitor de tela vem do cabeçalho da página.
 */
export function TournamentListSkeleton() {
  return (
    <Stack gap={7}>
      <SectionSkeleton tiles={3} />
      <TableSkeleton columns={4} />
    </Stack>
  );
}
