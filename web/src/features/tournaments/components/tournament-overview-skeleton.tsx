import { PageColumns } from "@/shared/ui/page-columns";
import { SectionSkeleton, TabLoading } from "@/shared/ui/skeleton";

/**
 * A Visão geral carregando, nas duas colunas dela: na principal, a situação
 * e o resumo do formato; na estreita, o ciclo do torneio.
 */
export function TournamentOverviewSkeleton({ label }: { label: string }) {
  return (
    <TabLoading label={label}>
      <PageColumns
        main={
          <>
            <SectionSkeleton rows={1} />
            <SectionSkeleton rows={6} />
          </>
        }
        aside={<SectionSkeleton rows={4} />}
      />
    </TabLoading>
  );
}
