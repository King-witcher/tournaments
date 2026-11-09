"use client";

import { Draggable } from "@carbon/icons-react";
import type { ComponentProps, Ref } from "react";

import { cn } from "@/shared/lib/cn";
import styles from "./drag-handle.module.scss";

type Props = Omit<ComponentProps<"button">, "children" | "ref"> & {
  /** Ocupa a altura da linha e tem a largura de um controle. */
  ref?: Ref<HTMLButtonElement>;
  /** O item desta alça é o que está sendo arrastado: some só o ícone, e o foco do teclado fica nela. */
  dragging?: boolean;
};

/**
 * A alça que começa um arrasto (o ícone de seis pontos do Carbon), com o
 * tamanho de um controle (48px no celular). É o único ponto de uma linha onde
 * o toque não rola a página, e o nome (`aria-label`) diz o item. Ponha nela o
 * `setActivatorNodeRef`, os `attributes` e os `listeners` do dnd-kit. Serve à
 * `SortableList` e a quem arrasta entre listas com `@dnd-kit/core` direto.
 */
export function DragHandle({ ref, dragging, className, ...rest }: Props) {
  return (
    <button
      ref={ref}
      type="button"
      className={cn(styles.handle, dragging && styles.dragging, className)}
      {...rest}
    >
      <Draggable />
    </button>
  );
}

/** A mesma alça, só desenhada: na imagem que acompanha o cursor. */
export function DragHandlePreview() {
  return (
    <span className={styles.handle}>
      <Draggable />
    </span>
  );
}
