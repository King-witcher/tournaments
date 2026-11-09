import { DndContext, useDraggable } from "@dnd-kit/core";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";

import { DragHandle, DragHandlePreview } from "./drag-handle";
import { DragImage } from "./drag-image";

const meta = {
  title: "Listas/DragHandle",
  component: DragHandle,
  args: { "aria-label": "Mover Lobo" },
} satisfies Meta<typeof DragHandle>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Passe o mouse, foque com o Tab ou aperte para ver os estados. */
export const Normal: Story = {};

/** O lugar do item que está sendo arrastado: só o ícone some. */
export const Dragging: Story = { args: { dragging: true } };

/** A alça desenhada na imagem que acompanha o cursor. */
export const Preview: Story = {
  render: () => <DragHandlePreview />,
};

function Item() {
  const { attributes, listeners, setNodeRef, setActivatorNodeRef, isDragging } =
    useDraggable({ id: "lobo" });
  return (
    <div ref={setNodeRef} style={{ display: "flex", alignItems: "center" }}>
      <DragHandle
        ref={setActivatorNodeRef}
        aria-label="Mover Lobo"
        dragging={isDragging}
        {...attributes}
        {...listeners}
      />
      Lobo
    </div>
  );
}

function ImageDemo() {
  const [active, setActive] = useState(false);
  return (
    <DndContext
      onDragStart={() => setActive(true)}
      onDragEnd={() => setActive(false)}
      onDragCancel={() => setActive(false)}
    >
      <Item />
      {active && <DragImage>Lobo</DragImage>}
    </DndContext>
  );
}

/** Arraste pela alça: a `DragImage` acompanha o cursor. */
export const WithDragImage: Story = {
  render: () => <ImageDemo />,
};
