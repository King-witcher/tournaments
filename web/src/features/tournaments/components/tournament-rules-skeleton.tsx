import { MapGridSkeleton } from "@/features/maps/components/map-grid-skeleton";
import { SkeletonText } from "@/shared/ui/carbon/skeleton-text";
import { Stack } from "@/shared/ui/carbon/stack";
import { SectionSkeleton, TabLoading } from "@/shared/ui/skeleton";
import styles from "./tournament-rules.module.scss";

/**
 * As Regras carregando, na largura de leitura delas: o texto gerado do
 * formato, em linhas, e embaixo a grade dos mapas do pool, que ocupa bem
 * mais que uma linha.
 */
export function TournamentRulesSkeleton({ label }: { label: string }) {
  return (
    <div className={styles.reading}>
      <TabLoading label={label}>
        <SectionSkeleton rows={6} />
        <div aria-hidden>
          <Stack gap={5}>
            <SkeletonText heading width="40%" />
            <MapGridSkeleton />
          </Stack>
        </div>
      </TabLoading>
    </div>
  );
}
