import { PageHeaderSkeleton, SectionSkeleton } from "@/shared/ui/skeleton";
import { FormationBoard } from "./formation-board";
import styles from "./formation-skeleton.module.scss";

/**
 * A página dos times carregando, na forma dela: o cabeçalho com a
 * explicação, o resumo e os sorteios, e a formação (os times em grade e os
 * reservas ao lado). `label` é lido pelo leitor de tela.
 */
export function FormationSkeleton({ label }: { label: string }) {
  return (
    <>
      <PageHeaderSkeleton meta label={label} />
      <SectionSkeleton rows={1} />
      <FormationBoard
        teams={
          <div aria-hidden className={styles.grid}>
            {[1, 2, 3, 4].map((n) => (
              <span key={n} className={styles.team} />
            ))}
          </div>
        }
        reserves={<span aria-hidden className={styles.reserves} />}
      />
    </>
  );
}
