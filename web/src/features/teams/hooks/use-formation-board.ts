import { useMemo, useRef, useState } from "react";

import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { RESERVES_ID, teamBlockId } from "../lib/focus";
import {
  canMove,
  type Destination,
  type Formation,
  type Spot,
  spotOf,
} from "../lib/formation";
import {
  applyEdit,
  applyEdits,
  draftBody,
  type FormationEdit,
  sameFormation,
} from "../lib/formation-draft";

/** Quem está sendo arrastado e de onde saiu. */
export type DraggedPlayer = { userId: number; from: Spot };

/**
 * O estado da edição da formação: a formação da API (a última que ela devolveu,
 * ou a da página quando ela recarrega) e, por cima, o que a organização mudou
 * e não salvou (`FormationEdit`). `formation` é o rascunho que a tela mostra:
 * mover, criar time e trocar capitão mudam só ele, na hora, e `save` grava o
 * rascunho inteiro. O tempo real só troca a formação de baixo, e as mudanças
 * são reaplicadas sobre ela.
 */
export function useFormationBoard(
  tenant: string,
  tournament: string,
  initial: Formation,
) {
  const [server, setServer] = useState(initial);
  // A página recarregou (outra pessoa mexeu, ou o tempo real): vale a dela.
  const [shown, setShown] = useState(initial);
  if (initial !== shown) {
    setShown(initial);
    setServer(initial);
  }
  const [edits, setEdits] = useState<FormationEdit[]>([]);
  const formation = useMemo(() => applyEdits(server, edits), [server, edits]);
  const dirty = edits.length > 0 && !sameFormation(server, formation);
  // O que foi desfeito à mão: sem isto, uma mudança da página reviveria as
  // edições que já se anularam.
  if (edits.length > 0 && !dirty) setEdits([]);

  const [saved, setSaved] = useState(false);
  const [drag, setDrag] = useState<DraggedPlayer>();
  const { run, pending, error } = useApiAction();
  const nextDraftId = useRef(0);

  function commit(edit: FormationEdit) {
    if (pending) return;
    const next = applyEdit(formation, edit);
    if (next === formation) return;
    setEdits([...edits, edit]);
    setSaved(false);
    const spot = spotOf(next, edit.userId);
    focusIfLost(typeof spot === "number" ? teamBlockId(spot) : RESERVES_ID);
  }

  function moveTo(userId: number, to: Destination) {
    if (to === "new") {
      nextDraftId.current -= 1;
      commit({ kind: "newTeam", userId, teamId: nextDraftId.current });
    } else {
      commit({ kind: "move", userId, to });
    }
  }

  function drop(player: DraggedPlayer, to: Destination) {
    if (canMove(formation, player.from, to)) moveTo(player.userId, to);
  }

  function makeCaptain(userId: number) {
    commit({ kind: "captain", userId });
  }

  function cancel() {
    setEdits([]);
    setSaved(false);
  }

  /** Uma resposta que substitui tudo (o sorteio): o rascunho se perde. */
  function replace(data: Formation) {
    setServer(data);
    setEdits([]);
    setSaved(false);
  }

  /** Uma resposta de fora do rascunho (renomear): ele continua por cima. */
  function adopt(data: Formation) {
    setServer(data);
  }

  function save() {
    if (!dirty || pending) return;
    run(
      () =>
        api.PUT("/tenants/{slug}/tournaments/{tournament}/teams/formation", {
          params: { path: { slug: tenant, tournament } },
          body: draftBody(formation),
        }),
      (data) => {
        setServer(data);
        setEdits([]);
        setSaved(true);
      },
    );
  }

  return {
    formation,
    drag,
    setDrag,
    drop,
    moveTo,
    makeCaptain,
    dirty,
    saved,
    save,
    cancel,
    replace,
    adopt,
    pending,
    error,
  };
}

export type FormationBoardState = ReturnType<typeof useFormationBoard>;
