import { SkeletonText } from "@/shared/ui/carbon/skeleton-text";
import { MapGrid } from "./map-grid";
import styles from "./map-grid-skeleton.module.scss";

/**
 * A grade de mapas carregando: `count` caixas com a imagem em 16:9 e a linha
 * do nome, na mesma grade (`MapGrid`), para a página não pular quando os
 * mapas chegam. Só visual: quem a usa já avisa que está carregando.
 */
export function MapGridSkeleton({ count = 7 }: { count?: number }) {
  return (
    <div aria-hidden>
      <MapGrid>
        {Array.from({ length: count }, (_, i) => (
          // biome-ignore lint/suspicious/noArrayIndexKey: caixas fixas, sem identidade.
          <div key={i} className={styles.item}>
            <span className={styles.image} />
            <SkeletonText width="60%" />
          </div>
        ))}
      </MapGrid>
    </div>
  );
}
