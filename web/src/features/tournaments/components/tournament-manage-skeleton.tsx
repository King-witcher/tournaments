import { PageColumns } from "@/shared/ui/page-columns";
import { Panel } from "@/shared/ui/panel";
import { SectionSkeleton, TabLoading } from "@/shared/ui/skeleton";

/**
 * A Organização carregando, nas duas colunas dela: o estado com o próximo
 * passo na principal e a ordem da chave na estreita.
 */
export function TournamentManageSkeleton({ label }: { label: string }) {
  return (
    <TabLoading label={label}>
      <PageColumns
        main={
          <Panel>
            <SectionSkeleton rows={2} />
          </Panel>
        }
        aside={
          <Panel>
            <SectionSkeleton rows={1} />
          </Panel>
        }
      />
    </TabLoading>
  );
}
