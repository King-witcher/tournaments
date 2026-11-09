import type { HTMLAttributes, Ref } from "react";

import { cn } from "@/shared/lib/cn";
import styles from "./formation-block.module.scss";

type Props = HTMLAttributes<HTMLElement> & {
  /** `section` para um time (tem título); `div` para a lista dos reservas e a área de time novo. */
  as?: "div" | "section";
  /** Um jogador arrastado está em cima e pode ser solto aqui. */
  highlight?: boolean;
  /** Borda tracejada: uma área só de soltar (o time novo). */
  dashed?: boolean;
  /** Para o bloco que recebe jogadores arrastados (`useDroppable`). */
  ref?: (node: HTMLElement | null) => void;
};

/**
 * Um bloco da formação dos times (um time, os reservas, a área de time novo),
 * no modelo do `Tile` do Carbon. Enquanto alguém é arrastado, o bloco onde
 * ele pode ser solto ganha o contorno da área de soltar do Carbon.
 */
export function FormationBlock({
  as: Tag = "div",
  highlight,
  dashed,
  className,
  ref,
  ...rest
}: Props) {
  return (
    <Tag
      ref={ref as Ref<HTMLDivElement>}
      className={cn(
        styles.block,
        dashed && styles.dashed,
        highlight && styles.highlight,
        className,
      )}
      {...rest}
    />
  );
}
