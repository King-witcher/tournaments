import { PageHeaderSkeleton, SectionSkeleton } from "@/shared/ui/skeleton";
import styles from "./match-skeleton.module.scss";

/**
 * A série carregando, na forma dela: o cabeçalho com a trilha, o indicador
 * dos passos, o passo à vista e os blocos fechados do fim. Vai direto no
 * `Page`.
 */
export function MatchSkeleton({ label }: { label: string }) {
  return (
    <>
      <PageHeaderSkeleton meta label={label} />
      <span aria-hidden className={styles.steps} />
      <SectionSkeleton rows={3} />
      <span aria-hidden className={styles.closed} />
    </>
  );
}
