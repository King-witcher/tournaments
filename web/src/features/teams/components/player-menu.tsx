"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import {
  type OverflowAction,
  OverflowActions,
} from "@/shared/ui/overflow-actions";
import { useMenuTrigger } from "../hooks/use-menu-trigger";
import type { Formation, Spot } from "../lib/formation";
import { MovePlayerModal } from "./move-player-modal";

type Props = {
  tenant: string;
  tournament: string;
  formation: Formation;
  userId: number;
  /** O apelido como aparece (já com "Conta excluída"). */
  name: string;
  /** Onde ele está agora. */
  from: Spot;
  /** Já é o capitão do time: sem "Tornar capitão". */
  captain: boolean;
  /** A API respondeu: a formação nova e quem mudou de lugar. */
  onDone: (formation: Formation, userId: number) => void;
};

/**
 * O ⋯ de um jogador na formação, a alternativa ao arrastar: "Mover para…"
 * (o `MovePlayerModal`) e, num time, "Tornar capitão". O erro do capitão fica
 * embaixo do menu; o de mover, no Modal.
 */
export function PlayerMenu({
  tenant,
  tournament,
  formation,
  userId,
  name,
  from,
  captain,
  onDone,
}: Props) {
  const t = useTranslations("Team");
  const { wrapper, trigger } = useMenuTrigger();
  const [moving, setMoving] = useState(false);
  const promote = useApiAction();

  const items: OverflowAction[] = [
    { label: t("move.open"), onSelect: () => setMoving(true) },
  ];
  if (typeof from === "number" && !captain) {
    items.push({
      label: t("makeCaptain"),
      onSelect: () =>
        promote.run(
          () =>
            api.PATCH("/tenants/{slug}/tournaments/{tournament}/teams/{team}", {
              params: { path: { slug: tenant, tournament, team: from } },
              body: { captain: userId },
            }),
          (data) => onDone(data, userId),
        ),
    });
  }

  return (
    <div ref={wrapper}>
      <ActionWithNote align="end" error={promote.error}>
        <OverflowActions
          items={items}
          pending={promote.pending}
          size="sm"
          label={t("playerActions", { name })}
        />
      </ActionWithNote>
      {moving && (
        <MovePlayerModal
          tenant={tenant}
          tournament={tournament}
          formation={formation}
          userId={userId}
          name={name}
          from={from}
          returnFocusTo={trigger}
          onClose={() => setMoving(false)}
          onDone={(data) => {
            setMoving(false);
            onDone(data, userId);
          }}
        />
      )}
    </div>
  );
}
