import { Panel } from "@/shared/ui/panel";
import { SectionSkeleton, TabLoading } from "@/shared/ui/skeleton";

/**
 * As Sessões carregando: o título e as linhas da lista (o cabeçalho dela e
 * duas sessões), na largura toda. `label`: o que o leitor de tela ouve;
 * `false` quando outra peça da página já diz.
 */
export function SessionsSkeleton({ label }: { label?: string | false }) {
  const list = (
    <Panel>
      <SectionSkeleton rows={3} />
    </Panel>
  );
  return label ? <TabLoading label={label}>{list}</TabLoading> : list;
}
