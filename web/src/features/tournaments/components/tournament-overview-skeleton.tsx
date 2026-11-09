import { PageColumns } from "@/shared/ui/page-columns";
import { Panel } from "@/shared/ui/panel";
import { SectionSkeleton, TabLoading } from "@/shared/ui/skeleton";

/**
 * A Visão geral carregando, nas duas colunas dela: na principal, a situação
 * e o resumo do formato; na estreita, o ciclo do torneio. Com os `Panel` da
 * seção, em volta do resumo e do ciclo.
 */
export function TournamentOverviewSkeleton({ label }: { label: string }) {
  return (
    <TabLoading label={label}>
      <PageColumns
        main={
          <>
            <SectionSkeleton rows={1} />
            <Panel>
              <SectionSkeleton rows={6} />
            </Panel>
          </>
        }
        aside={
          <Panel>
            <SectionSkeleton rows={4} />
          </Panel>
        }
      />
    </TabLoading>
  );
}
