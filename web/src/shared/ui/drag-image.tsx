"use client";

import { DragOverlay } from "@dnd-kit/core";
import type { ReactNode } from "react";
import { createPortal } from "react-dom";

import styles from "./drag-image.module.scss";

/** Acima do Modal do Carbon (`z("modal")`, 9000). */
const OVERLAY_Z_INDEX = 9001;

/**
 * A imagem do que está sendo arrastado, que acompanha o cursor ou o dedo, no
 * tamanho do item de origem e com a sombra do Carbon. Fica no `body` (o Modal
 * do Carbon tem `transform`, que prenderia o `position: fixed`) e é `inert`.
 * Monte só enquanto arrasta (`{active && <DragImage>…</DragImage>}`), dentro
 * do `DndContext`, com o conteúdo sem foco nem clique e sem `id` fixo: o item
 * de origem continua na página.
 */
export function DragImage({ children }: { children: ReactNode }) {
  return createPortal(
    <DragOverlay
      className={styles.overlay}
      zIndex={OVERLAY_Z_INDEX}
      dropAnimation={null}
    >
      <div className={styles.image} inert>
        {children}
      </div>
    </DragOverlay>,
    document.body,
  );
}
