"use client";

import { useTranslations } from "next-intl";

import { ChampionsStep } from "@/features/matches/components/champions-step";
import { MapsStep } from "@/features/matches/components/maps-step";
import { VetoReady } from "@/features/matches/components/veto-ready";
import { VetoStepsFrame } from "@/features/matches/components/veto-steps-frame";
import { useFollowedStep } from "@/features/matches/hooks/use-followed-step";
import { useVetoBoard } from "@/features/matches/hooks/use-veto-board";
import { useVetoRoom } from "../hooks/use-veto-room";
import { roomAsks, roomPhase, roomSteps } from "../lib/room-steps";
import { VetoRoomResult } from "./veto-room-result";

/**
 * Os passos da sala (Pronto, Mapas, Campeões e Resultado; só os que ela tem),
 * os mesmos da série, um por vez. O indicador é a navegação: os passos feitos
 * e o atual abrem.
 */
export function VetoRoomSteps() {
  const t = useTranslations("Match.steps");
  const tRoom = useTranslations("VetoRoom");
  const { veto, pending } = useVetoBoard();
  const { room, isMine, clearMine } = useVetoRoom();
  // Sempre há veto na sala; o `VetoBoard` também serve à série, que pode não ter.
  const shownVeto = veto ?? room.veto;
  const phase = roomPhase(shownVeto);
  const view = useFollowedStep({
    home: phase,
    pending,
    isMine,
    clearMine,
    asks: (key) => roomAsks(shownVeto, key),
  });
  const steps = roomSteps(shownVeto);
  const current = steps.indexOf(phase);
  const index = steps.indexOf(view.shown);
  const done = index < current;
  const rule = room.no_repeat
    ? { who: room.no_repeat, scope: "series" as const }
    : null;

  return (
    <VetoStepsFrame
      steps={steps.map((key, i) => ({
        key,
        label: key === "result" ? tRoom("result") : t(key),
        secondaryLabel:
          i === current && key !== "result" ? t("now") : undefined,
      }))}
      current={current}
      label={tRoom("stepsLabel")}
      onSelect={(i) => view.show(steps[i])}
      selected={index}
    >
      {view.shown === "ready" && <VetoReady done={done} room />}
      {view.shown === "maps" && <MapsStep done={done} />}
      {view.shown === "champions" && <ChampionsStep rule={rule} done={done} />}
      {view.shown === "result" && <VetoRoomResult />}
    </VetoStepsFrame>
  );
}
