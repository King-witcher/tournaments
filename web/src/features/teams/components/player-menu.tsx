"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import {
  type OverflowAction,
  OverflowActions,
} from "@/shared/ui/overflow-actions";
import { useMenuTrigger } from "../hooks/use-menu-trigger";
import type { Destination, Formation, Spot } from "../lib/formation";
import { MovePlayerModal } from "./move-player-modal";

type Props = {
  formation: Formation;
  /** O apelido como aparece (já com "Conta excluída"). */
  name: string;
  /** Onde ele está agora. */
  from: Spot;
  /** Já é o capitão do time: sem "Tornar capitão". */
  captain: boolean;
  onMove: (to: Destination) => void;
  onMakeCaptain: () => void;
};

/**
 * O ⋯ de um jogador na formação, a alternativa ao arrastar: "Mover para…"
 * (o `MovePlayerModal`) e, num time, "Tornar capitão". Os dois mudam o
 * rascunho da formação, na hora.
 */
export function PlayerMenu({
  formation,
  name,
  from,
  captain,
  onMove,
  onMakeCaptain,
}: Props) {
  const t = useTranslations("Team");
  const { wrapper, trigger } = useMenuTrigger();
  const [moving, setMoving] = useState(false);

  const items: OverflowAction[] = [
    { label: t("move.open"), onSelect: () => setMoving(true) },
  ];
  if (typeof from === "number" && !captain) {
    items.push({ label: t("makeCaptain"), onSelect: onMakeCaptain });
  }

  return (
    <div ref={wrapper}>
      <OverflowActions
        items={items}
        size="sm"
        label={t("playerActions", { name })}
      />
      {moving && (
        <MovePlayerModal
          formation={formation}
          name={name}
          from={from}
          returnFocusTo={trigger}
          onClose={() => setMoving(false)}
          onMove={(to) => {
            setMoving(false);
            onMove(to);
          }}
        />
      )}
    </div>
  );
}
