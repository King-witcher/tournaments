"use client";

import { useTranslations } from "next-intl";
import type { ReactNode } from "react";
import { shownNickname } from "@/features/members/lib/nickname";
import { useVetoBoard } from "../hooks/use-veto-board";
import {
  currentGame,
  isActing,
  isWindow,
  nextMapPickGame,
  turnKey,
  turnRule,
} from "../lib/veto";
import { VetoTurnBar } from "./veto-turn-bar";

/**
 * De quem é a vez no veto, o que ele faz e o tempo que falta pela hora da API
 * (no fim, a escolha é sorteada: a página recarrega para mostrar o sorteio; na
 * janela simultânea, o tempo da janela). Nos picks alternados, quem já escolheu
 * naquela partida vê quem do lado dele falta, e quem a regra do torneio não deixa
 * agir vê que é o capitão quem faz. Some sem vez (o passo atual mostra o resto). No instante entre um passo marcado na
 * hora e a resposta, quando a vez seguinte é de outra fase (`sides` vazio),
 * fica no lugar dizendo que enviou, para a grade não pular embaixo do dedo.
 */
export function VetoTurn() {
  const t = useTranslations("Match.veto");
  const tMember = useTranslations("Member");
  const { veto, now, mine, names, refresh } = useVetoBoard();
  const turn = veto?.turn;
  if (!veto || !turn || veto.status !== "in_progress") return null;
  const game = currentGame(veto);
  const teamPick = turn.action === "team_pick";
  const window = isWindow(turn);
  const allowed = veto.can_act[turnRule(turn)];
  const sideTurn = mine != null && turn.sides.includes(mine);
  const ours = isActing(veto, mine);
  const forTeam = veto.can_act.pick_for_team;
  const blind = veto.pick_order === "blind";
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
  } else if (sideTurn && !allowed) {
    // A vez é do lado dele, e a regra deixa com o capitão.
    const rule = turnRule(turn);
    sentence = t(`captainActs.${turn.action === "vote" ? "vote" : rule}`);
  } else if (window) {
    sentence = blind
      ? t.rich(
          forTeam && ours
            ? "blindPickCaptain"
            : ours
              ? "blindPickYours"
              : veto.picks_by_captain
                ? "blindPickCaptains"
                : "blindPick",
          { b },
        )
      : t.rich(ours ? "teamPickYours" : "teamPick", { b });
  } else if (teamPick && !ours && sideTurn) {
    // É a vez do lado dele, e ele já escolheu nesta partida.
    const left = veto.missing
      .filter((p) => p.side === mine && p.games.includes(turn.pick_for ?? game))
      .map((p) => shownNickname(p.nickname, tMember("deletedAccount")));
    sentence = t("pickedWaitingMate", {
      game: turn.pick_for ?? game,
      names: left.join(", "),
    });
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
      note={t(window ? "teamDeadline" : "deadline")}
      acting={ours}
      deadline={veto.deadline ?? null}
      seconds={(window && veto.team_pick_seconds) || veto.step_seconds}
      serverNow={now}
      // O sorteio é feito pela API, que ainda aceita o clique por 2 s depois
      // do prazo: recarrega depois disso para mostrar.
      onExpire={() => setTimeout(refresh, 2500)}
    />
  );
}
