"use client";

import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { useVetoBoard } from "../hooks/use-veto-board";
import { currentGame, nextMapPickGame, turnKey } from "../lib/veto";
import { VetoTurnBar } from "./veto-turn-bar";

/**
 * De quem é a vez no veto, o que ele faz e o tempo que falta pela hora da API
 * (no fim, a escolha é sorteada: a página recarrega para mostrar o sorteio; na
 * janela simultânea do TDM, o tempo da janela). Some sem vez (o passo atual
 * mostra o resto). No instante entre um passo marcado na
 * hora e a resposta, quando a vez seguinte é de outra fase (`sides` vazio),
 * fica no lugar dizendo que enviou, para a grade não pular embaixo do dedo.
 */
export function VetoTurn() {
  const t = useTranslations("Match.veto");
  const { veto, now, mine, names, refresh } = useVetoBoard();
  const turn = veto?.turn;
  if (!veto || !turn || veto.status !== "in_progress") return null;
  const game = currentGame(veto);
  const teamPick = turn.action === "team_pick";
  const ours = teamPick
    ? veto.picks.length > 0
    : mine != null && turn.sides.includes(mine);
  // O verbo em negrito: é ele que muda de "banir" para "escolher".
  const b = (verb: ReactNode) => <strong>{verb}</strong>;
  const action = () =>
    t.rich(`action.${turnKey(turn)}`, {
      b,
      game:
        turn.pick_for ??
        (turn.target === "map" && turn.action === "pick"
          ? nextMapPickGame(veto)
          : game),
    });

  let sentence: ReactNode;
  if (turn.sides.length === 0) {
    sentence = t("sending");
  } else if (teamPick) {
    sentence = t.rich(ours ? "teamPickYours" : "teamPick", { b });
  } else if (turn.sides.length === 2) {
    sentence = t.rich("bothVote", { game, b });
  } else if (ours) {
    sentence = t.rich("yourTurn", { action });
  } else if (turn.action === "vote" && mine) {
    sentence = t("votedWaiting", { name: names[turn.sides[0]] });
  } else {
    sentence = t.rich("theirTurn", { name: names[turn.sides[0]], action });
  }

  return (
    <VetoTurnBar
      sentence={sentence}
      note={t(teamPick ? "teamDeadline" : "deadline")}
      acting={ours}
      deadline={veto.deadline ?? null}
      seconds={(teamPick && veto.team_pick_seconds) || veto.step_seconds}
      serverNow={now}
      // O sorteio é feito pela API, que ainda aceita o clique por 2 s depois
      // do prazo: recarrega depois disso para mostrar.
      onExpire={() => setTimeout(refresh, 2500)}
    />
  );
}
