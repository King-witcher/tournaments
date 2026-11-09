"use client";

import { Draggable } from "@carbon/icons-react";
import {
  type Announcements,
  closestCenter,
  DndContext,
  type DragEndEvent,
  DragOverlay,
  KeyboardSensor,
  PointerSensor,
  type UniqueIdentifier,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { useTranslations } from "next-intl";
import { type ReactNode, useEffect, useId, useRef, useState } from "react";
import { createPortal } from "react-dom";

import { cn } from "@/shared/lib/cn";
import styles from "./sortable-list.module.scss";

/** O quanto o ponteiro anda para o arrasto começar; um toque parado na alça não arrasta. */
const DRAG_START_DISTANCE = 4;

/** Acima do Modal do Carbon (`z("modal")`, 9000). */
const OVERLAY_Z_INDEX = 9001;

type SortableItem = { id: UniqueIdentifier };

type Props<T extends SortableItem> = {
  /** Os itens na ordem de agora. O `id` é estável: nunca o índice. */
  items: T[];
  /** O nome do item, em texto: vai no nome da alça e nos anúncios do leitor de tela. */
  itemLabel: (item: T) => string;
  /** O conteúdo da linha, sem a alça. Desenha também a imagem que acompanha o cursor, sem foco nem clique. */
  renderItem: (item: T, index: number) => ReactNode;
  /** A ordem nova, ao soltar numa posição diferente. */
  onChange: (items: T[]) => void;
  /** Só consulta (o torneio já começou): sem a alça. */
  disabled?: boolean;
};

/**
 * Lista que se reordena arrastando pela alça, com mouse, toque ou teclado
 * (Espaço ou Enter pega, setas movem, Espaço ou Enter solta, Esc cancela). Os
 * anúncios do leitor de tela vêm traduzidos. Controlada: quem usa guarda a
 * ordem e recebe a nova em `onChange`.
 *
 * Só a alça começa o arrasto: no celular, o resto da linha continua rolando a
 * página, e botões e links dentro de `renderItem` funcionam normalmente. Serve
 * dentro de `PassiveModal`, `ConfirmModal` e `FormModal`: a imagem do item fica
 * acima do Modal, o corpo dele rola sozinho perto da borda e Esc durante o
 * arrasto cancela o arrasto sem fechar o Modal.
 *
 * Regras: `renderItem` não chama hook (ponha o que tem estado num componente)
 * e não usa `id` fixo no HTML, porque a linha aparece duas vezes durante o
 * arrasto. Para mover entre listas, use `@dnd-kit/core` direto.
 *
 * @example
 * <SortableList
 *   items={teams}
 *   itemLabel={(team) => team.name}
 *   renderItem={(team) => <TeamSeedRow team={team} />}
 *   onChange={setTeams}
 *   disabled={started}
 * />
 */
export function SortableList<T extends SortableItem>({
  items,
  itemLabel,
  renderItem,
  onChange,
  disabled,
}: Props<T>) {
  const t = useTranslations("Common.sortableList");
  // Sem `id`, o dnd-kit numera o `aria-describedby` por um contador do módulo,
  // que no servidor não bate com o do navegador.
  const id = useId();
  const [activeId, setActiveId] = useState<UniqueIdentifier>();
  // Logo depois de pegar, o dnd-kit avisa que o item está sobre o próprio lugar;
  // o leitor de tela só lê o último aviso, e esse tomaria o lugar do "Pegou".
  const lastOver = useRef<UniqueIdentifier>(undefined);
  useEscapeCancelsOnlyTheDrag(activeId !== undefined);
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: { distance: DRAG_START_DISTANCE },
    }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    }),
  );

  const indexOf = (itemId: UniqueIdentifier | undefined) =>
    items.findIndex((item) => item.id === itemId);
  const activeIndex = indexOf(activeId);
  const activeItem: T | undefined = items[activeIndex];

  /** O anúncio de `key` para o item `moving`, que está (ou estaria) no lugar de `at`. */
  function announce(
    key: "picked" | "moved" | "dropped" | "cancelled",
    moving: UniqueIdentifier,
    at: UniqueIdentifier | undefined,
  ) {
    const item: T | undefined = items[indexOf(moving)];
    const position = indexOf(at);
    if (!item || position === -1) return;
    return t(key, {
      item: itemLabel(item),
      position: position + 1,
      total: items.length,
    });
  }

  const announcements: Announcements = {
    onDragStart: ({ active }) => {
      lastOver.current = active.id;
      return announce("picked", active.id, active.id);
    },
    onDragOver: ({ active, over }) => {
      if (over?.id === lastOver.current) return;
      lastOver.current = over?.id;
      return announce("moved", active.id, over?.id);
    },
    onDragEnd: ({ active, over }) => announce("dropped", active.id, over?.id),
    onDragCancel: ({ active }) => announce("cancelled", active.id, active.id),
  };

  function handleDragEnd({ active, over }: DragEndEvent) {
    setActiveId(undefined);
    const from = indexOf(active.id);
    const to = indexOf(over?.id);
    if (from !== -1 && to !== -1 && from !== to) {
      onChange(arrayMove(items, from, to));
    }
  }

  return (
    <DndContext
      id={id}
      sensors={sensors}
      collisionDetection={closestCenter}
      accessibility={{
        announcements,
        screenReaderInstructions: { draggable: t("instructions") },
      }}
      onDragStart={({ active }) => setActiveId(active.id)}
      onDragCancel={() => setActiveId(undefined)}
      onDragEnd={handleDragEnd}
    >
      <SortableContext
        items={items}
        strategy={verticalListSortingStrategy}
        disabled={disabled}
      >
        <ul className={styles.list}>
          {items.map((item, index) => (
            <SortableRow
              key={item.id}
              id={item.id}
              label={itemLabel(item)}
              disabled={disabled}
            >
              {renderItem(item, index)}
            </SortableRow>
          ))}
        </ul>
      </SortableContext>
      {/* No body: o Modal do Carbon tem `transform`, que prenderia o `position: fixed` da imagem. */}
      {activeItem &&
        createPortal(
          <DragOverlay
            className={styles.overlay}
            zIndex={OVERLAY_Z_INDEX}
            dropAnimation={null}
          >
            <div className={cn(styles.row, styles.dragged)} inert>
              <span className={styles.handle}>
                <Draggable />
              </span>
              <div className={styles.content}>
                {renderItem(activeItem, activeIndex)}
              </div>
            </div>
          </DragOverlay>,
          document.body,
        )}
    </DndContext>
  );
}

type RowProps = {
  id: UniqueIdentifier;
  label: string;
  disabled?: boolean;
  children: ReactNode;
};

function SortableRow({ id, label, disabled, children }: RowProps) {
  const t = useTranslations("Common.sortableList");
  const {
    attributes,
    listeners,
    setNodeRef,
    setActivatorNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({
    id,
    // O movimento vem do `.row` (os tokens do Carbon e o `prefers-reduced-motion`).
    transition: null,
    attributes: { roleDescription: t("roleDescription") },
  });

  return (
    <li
      ref={setNodeRef}
      className={cn(styles.row, isDragging && styles.placeholder)}
      // A posição muda a cada quadro do arrasto: não cabe numa classe.
      style={{ transform: CSS.Translate.toString(transform), transition }}
    >
      {!disabled && (
        <button
          ref={setActivatorNodeRef}
          type="button"
          className={styles.handle}
          aria-label={t("handle", { item: label })}
          {...attributes}
          {...listeners}
        >
          <Draggable />
        </button>
      )}
      <div className={styles.content}>{children}</div>
    </li>
  );
}

/**
 * Durante o arrasto, Esc só cancela o arrasto. O Modal do Carbon escuta o Esc
 * no `document`, antes dos sensores do dnd-kit, e fecharia junto. Por isso o Esc
 * de verdade para aqui, e o sensor recebe um com só o `code`, que é o que ele
 * lê; o Modal lê o `key`.
 */
function useEscapeCancelsOnlyTheDrag(dragging: boolean) {
  useEffect(() => {
    if (!dragging) return;
    function onKeyDown(event: KeyboardEvent) {
      if (event.key !== "Escape") return;
      event.stopPropagation();
      document.dispatchEvent(new KeyboardEvent("keydown", { code: "Escape" }));
    }
    window.addEventListener("keydown", onKeyDown, { capture: true });
    return () =>
      window.removeEventListener("keydown", onKeyDown, { capture: true });
  }, [dragging]);
}
