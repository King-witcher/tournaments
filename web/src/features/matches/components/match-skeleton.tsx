import { PageHeaderSkeleton, SectionSkeleton } from "@/shared/ui/skeleton";
import styles from "./match-skeleton.module.scss";

/**
 * A partida carregando, na forma dela: o cabeçalho com a trilha, o placar,
 * a lista de mapas e os blocos fechados do fim. Vai direto no `Page`.
 */
export function MatchSkeleton({ label }: { label: string }) {
  return (
    <>
      <PageHeaderSkeleton meta label={label} />
      <span aria-hidden className={styles.board} />
      <SectionSkeleton rows={3} />
      <span aria-hidden className={styles.closed} />
    </>
  );
}
