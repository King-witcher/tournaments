import { TableSkeleton } from "@/shared/ui/skeleton";
import styles from "./bracket-skeleton.module.scss";

/** Partidas por rodada no esboço: uma chave de oito. */
const ROUNDS = [4, 2, 1];

/**
 * A aba Chave carregando, na forma dela: o título, as rodadas com os cartões
 * e a tabela da classificação. Leva o "Carregando a chave…" para o leitor de
 * tela (o cabeçalho do torneio já está na página).
 */
export function BracketSkeleton({ label }: { label: string }) {
  return (
    <div className={styles.skeleton}>
      <span role="status" className={styles.srOnly}>
        {label}
      </span>
      <span aria-hidden className={styles.title} />
      <div aria-hidden className={styles.rounds}>
        {ROUNDS.map((count) => (
          <div key={count} className={styles.round}>
            {Array.from({ length: count }, (_, i) => (
              // biome-ignore lint/suspicious/noArrayIndexKey: blocos fixos, sem identidade.
              <span key={i} className={styles.card} />
            ))}
          </div>
        ))}
      </div>
      <span aria-hidden className={styles.title} />
      <TableSkeleton columns={3} rows={4} />
    </div>
  );
}
