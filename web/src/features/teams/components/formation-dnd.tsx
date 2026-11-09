"use client";

import {
  type Announcements,
  type CollisionDetection,
  DndContext,
  KeyboardSensor,
  PointerSensor,
  pointerWithin,
  rectIntersection,
  type UniqueIdentifier,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import { useTranslations } from "next-intl";
import { type ReactNode, useId } from "react";

import { shownNickname } from "@/features/members/lib/nickname";
import { cn } from "@/shared/lib/cn";
import { DragHandlePreview } from "@/shared/ui/drag-handle";
import { DragImage } from "@/shared/ui/drag-image";
import type { FormationBoardState } from "../hooks/use-formation-board";
import { useTeamName } from "../hooks/use-team-name";
import { nextDropZone } from "../lib/drop-zones";
import { type Destination, playerOf, spotOf } from "../lib/formation";
import styles from "./formation-dnd.module.scss";
import { PlayerLine } from "./player-line";

/** O quanto o ponteiro anda para o arrasto começar; um toque parado na alça não arrasta. */
const DRAG_START_DISTANCE = 4;

/**
 * Com o ponteiro, vale o bloco embaixo dele; no teclado, sem ponteiro, o que
 * a imagem cobre (ela vai para o meio do destino).
 */
const collision: CollisionDetection = (args) =>
  args.pointerCoordinates ? pointerWithin(args) : rectIntersection(args);

type Props = { board: FormationBoardState; children: ReactNode };

/**
 * O arrastar de jogadores entre os times, a reserva e o time novo, pela alça,
 * com mouse, toque ou teclado (Espaço ou Enter pega, setas levam a outro bloco,
 * Espaço ou Enter solta, Esc cancela), com anúncios traduzidos. A imagem que
 * acompanha o cursor é só a linha do jogador. Fica em volta dos blocos;
 * cada jogador usa `useDraggable` e cada bloco, `useFormationDrop`.
 */
export function FormationDnd({ board, children }: Props) {
  const t = useTranslations("Team.drag");
  const tTeam = useTranslations("Team");
  const tMember = useTranslations("Member");
  const teamName = useTeamName();
  // Sem `id`, o dnd-kit numera o `aria-describedby` por um contador do módulo,
  // que no servidor não bate com o do navegador.
  const id = useId();
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: { distance: DRAG_START_DISTANCE },
    }),
    useSensor(KeyboardSensor, { coordinateGetter: nextDropZone }),
  );
  const { formation, drag, setDrag, drop } = board;

  function name(userId: UniqueIdentifier) {
    const player = playerOf(formation, Number(userId));
    return shownNickname(player?.nickname ?? "", tMember("deletedAccount"));
  }

  function destination(to: UniqueIdentifier) {
    if (to === "reserve") return tTeam("move.reserve");
    if (to === "new") return t("newTeam");
    const team = formation.teams.find((candidate) => candidate.id === to);
    return team ? teamName(team) : "";
  }

  const announcements: Announcements = {
    onDragStart: ({ active }) => t("picked", { name: name(active.id) }),
    onDragOver: ({ active, over }) =>
      over
        ? t("over", { name: name(active.id), target: destination(over.id) })
        : t("away", { name: name(active.id) }),
    onDragEnd: ({ active, over }) =>
      over
        ? t("dropped", { name: name(active.id), target: destination(over.id) })
        : t("notDropped", { name: name(active.id) }),
    onDragCancel: ({ active }) => t("cancelled", { name: name(active.id) }),
  };

  const dragged = drag && playerOf(formation, drag.userId);

  return (
    <DndContext
      id={id}
      sensors={sensors}
      collisionDetection={collision}
      accessibility={{
        announcements,
        screenReaderInstructions: { draggable: t("instructions") },
      }}
      onDragStart={({ active }) => {
        const userId = Number(active.id);
        const from = spotOf(formation, userId);
        if (from !== undefined) setDrag({ userId, from });
      }}
      onDragCancel={() => setDrag(undefined)}
      onDragEnd={({ over }) => {
        setDrag(undefined);
        if (drag && over) drop(drag, over.id as Destination);
      }}
    >
      {/* Enquanto arrasta, a dica dos botões do caminho não abre. */}
      <div className={cn(drag && styles.dragging)}>{children}</div>
      {dragged && (
        <DragImage>
          <ul>
            <PlayerLine
              tenant=""
              userId={dragged.user_id}
              nickname={dragged.nickname}
              avatarUrl={dragged.avatar_url}
              left={dragged.left}
              handle={<DragHandlePreview />}
            />
          </ul>
        </DragImage>
      )}
    </DndContext>
  );
}
