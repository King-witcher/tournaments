import type { Ref } from "react";

import { cn } from "@/shared/lib/cn";
import styles from "./user-text.module.scss";

type Props = {
  children: string;
  /**
   * Corta em duas linhas, com reticências no fim: um resumo com "Ver mais"
   * ao lado. Quem usa mede se cortou (`scrollHeight` maior que `clientHeight`).
   */
  clamp?: boolean;
  id?: string;
  ref?: Ref<HTMLDivElement>;
};

/**
 * Texto escrito por usuário (descrição, regras), sem formatação: as quebras
 * de linha ficam e o resto sai como texto (nada de HTML nem Markdown).
 */
export function UserText({ children, clamp, id, ref }: Props) {
  return (
    <div id={id} ref={ref} className={cn(styles.text, clamp && styles.clamp)}>
      {children}
    </div>
  );
}
