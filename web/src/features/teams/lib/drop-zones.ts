import type { KeyboardCoordinateGetter } from "@dnd-kit/core";

/** Para onde cada seta leva. */
const DIRECTIONS: Record<string, { x: number; y: number }> = {
  ArrowRight: { x: 1, y: 0 },
  ArrowLeft: { x: -1, y: 0 },
  ArrowDown: { x: 0, y: 1 },
  ArrowUp: { x: 0, y: -1 },
};

type Rect = { top: number; left: number; width: number; height: number };

function centerOf({ top, left, width, height }: Rect) {
  return { x: left + width / 2, y: top + height / 2 };
}

/**
 * O arrasto pelo teclado: cada seta leva ao meio do destino mais perto naquela
 * direção (dando preferência a quem está em linha reta), em vez de andar alguns
 * pixels. Só contam os destinos que aceitam o jogador.
 */
export const nextDropZone: KeyboardCoordinateGetter = (
  event,
  { context: { collisionRect, droppableRects, droppableContainers } },
) => {
  const direction = DIRECTIONS[event.code];
  if (!direction) return undefined;
  // Sem isto, a seta também rola a página.
  event.preventDefault();
  if (!collisionRect) return undefined;

  const from = centerOf(collisionRect);
  let closest: Rect | undefined;
  let closestScore = Number.POSITIVE_INFINITY;
  for (const container of droppableContainers.getEnabled()) {
    const rect = droppableRects.get(container.id);
    if (!rect) continue;
    const to = centerOf(rect);
    // O dnd-kit só rola a página na direção da seta. Para os lados, um destino
    // com o meio fora da tela não apareceria: ali se chega pelas setas de cima
    // e baixo.
    if (direction.x !== 0 && (to.y < 0 || to.y > window.innerHeight)) continue;
    const along = (to.x - from.x) * direction.x + (to.y - from.y) * direction.y;
    // O destino de agora tem o centro no mesmo ponto.
    if (along <= 0) continue;
    const across =
      Math.abs(to.x - from.x) * Math.abs(direction.y) +
      Math.abs(to.y - from.y) * Math.abs(direction.x);
    const score = along + 2 * across;
    if (score < closestScore) {
      closest = rect;
      closestScore = score;
    }
  }
  if (!closest) return undefined;
  return {
    x: closest.left + (closest.width - collisionRect.width) / 2,
    y: closest.top + (closest.height - collisionRect.height) / 2,
  };
};
