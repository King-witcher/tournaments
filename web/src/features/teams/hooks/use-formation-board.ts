import { type DragEvent, useState } from "react";

import { focusIfLost } from "@/shared/lib/focus";
import { RESERVES_ID, teamBlockId } from "../lib/focus";
import {
  canMove,
  type Destination,
  type Formation,
  type Spot,
  spotOf,
} from "../lib/formation";
import { useMovePlayer } from "./use-move-player";

/** O tipo do que o arrastar põe na transferência: o `user_id` do jogador. */
const DRAG_TYPE = "text/plain";

/**
 * O estado da edição da formação: a formação de agora (a última que a API
 * devolveu, ou a da página quando ela recarrega) e o arrastar entre os
 * blocos. `dragProps` vai no `<li>` do jogador; `dropProps`, no bloco que
 * recebe (um time, a reserva, o time novo). Só o bloco onde dá para soltar
 * aceita o arrasto e fica destacado enquanto o jogador está em cima. O
 * resultado (andamento e erro) aparece no bloco de destino.
 */
export function useFormationBoard(
  tenant: string,
  tournament: string,
  initial: Formation,
) {
  const [formation, setFormation] = useState(initial);
  // A página recarregou (outra pessoa mexeu, ou o tempo real): vale a dela.
  const [shown, setShown] = useState(initial);
  if (initial !== shown) {
    setShown(initial);
    setFormation(initial);
  }
  const [drag, setDrag] = useState<{ userId: number; from: Spot }>();
  const [over, setOver] = useState<Destination>();
  const [target, setTarget] = useState<Destination>();
  const moving = useMovePlayer(tenant, tournament);

  /** Grava a formação nova e leva o foco ao bloco onde o jogador ficou. */
  function settle(data: Formation, userId: number) {
    setFormation(data);
    const spot = spotOf(data, userId);
    focusIfLost(typeof spot === "number" ? teamBlockId(spot) : RESERVES_ID);
  }

  function end() {
    setDrag(undefined);
    setOver(undefined);
  }

  function dragProps(userId: number, from: Spot) {
    return {
      draggable: !moving.pending,
      onDragStart(event: DragEvent<HTMLElement>) {
        event.dataTransfer.effectAllowed = "move";
        event.dataTransfer.setData(DRAG_TYPE, String(userId));
        setDrag({ userId, from });
      },
      onDragEnd: end,
    };
  }

  function dropProps(to: Destination) {
    const allowed = drag !== undefined && canMove(formation, drag.from, to);
    return {
      highlight: allowed && over === to,
      busy: moving.pending && target === to,
      error: target === to ? moving.error : undefined,
      onDragOver(event: DragEvent<HTMLElement>) {
        if (!allowed) return;
        // Sem o preventDefault, o navegador não deixa soltar aqui.
        event.preventDefault();
        event.dataTransfer.dropEffect = "move";
        setOver(to);
      },
      onDragLeave(event: DragEvent<HTMLElement>) {
        // Passar por cima de um filho do bloco também dispara o leave.
        if (event.currentTarget.contains(event.relatedTarget as Node | null)) {
          return;
        }
        setOver((current) => (current === to ? undefined : current));
      },
      onDrop(event: DragEvent<HTMLElement>) {
        event.preventDefault();
        end();
        if (!allowed || !drag) return;
        setTarget(to);
        moving.move(drag.userId, drag.from, to, (data) =>
          settle(data, drag.userId),
        );
      },
    };
  }

  return { formation, replace: setFormation, settle, dragProps, dropProps };
}

export type FormationBoardState = ReturnType<typeof useFormationBoard>;
