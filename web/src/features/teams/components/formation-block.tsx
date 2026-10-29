import type { HTMLAttributes } from "react";

import { cn } from "@/shared/lib/cn";
import styles from "./formation-block.module.scss";

type Props = HTMLAttributes<HTMLElement> & {
  /** `section` para um time (tem título); `div` para a lista dos reservas e a área de time novo. */
  as?: "div" | "section";
  /** Um jogador arrastado está em cima e pode ser solto aqui. */
  highlight?: boolean;
  /** Borda tracejada: uma área só de soltar (o time novo). */
  dashed?: boolean;
};

/**
 * Um bloco da formação dos times (um time, os reservas, a área de time novo):
 * a borda separa um grupo de jogadores do outro e, enquanto alguém é
 * arrastado, o bloco onde ele pode ser solto fica destacado. Só tokens: o
 * fundo e a borda de seleção do tema.
 */
export function FormationBlock({
  as: Tag = "div",
  highlight,
  dashed,
  className,
  ...rest
}: Props) {
  return (
    <Tag
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
