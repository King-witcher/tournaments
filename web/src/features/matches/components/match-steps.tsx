"use client";

import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { useFollowedStep } from "../hooks/use-followed-step";
import { useLiveMatch } from "../hooks/use-live-match";
import { useStepItems } from "../hooks/use-step-items";
import { asks, type StepKey, seriesSteps, stepGame } from "../lib/match-steps";
import { ChampionsStep } from "./champions-step";
import { CopyVetoResult } from "./copy-veto-result";
import { MapsStep } from "./maps-step";
import { VetoReady } from "./veto-ready";
import { VetoStepsFrame } from "./veto-steps-frame";

type Props = {
  rule: components["schemas"]["NoRepeatRule"] | null;
  /** Cada partida que abre (as enviadas e a da vez), desenhada no servidor. */
  games: Partial<Record<StepKey, ReactNode>>;
};

/**
 * Os passos da série (Pronto, Mapas, Campeões e cada partida; só os que o
 * torneio tem) e o passo à vista, um por vez. Abre no que pede uma ação de
 * quem vê, senão no atual; o indicador é a navegação (os passos feitos e o
 * atual abrem, os seguintes não), e a tela segue sozinha como o
 * `useFollowedStep` diz. As partidas ficam montadas mesmo escondidas (o
 * formulário de envio não perde o que já foi preenchido). Com o veto acabado,
 * o resultado dele para copiar fica entre o indicador e o passo.
 */
export function MatchSteps({ rule, games }: Props) {
  const t = useTranslations("Match.steps");
  const stepItems = useStepItems();
  const { detail, pending, isMine, clearMine } = useLiveMatch();
  const flow = seriesSteps(detail);
  const view = useFollowedStep({
    home: flow.home,
    pending,
    isMine,
    clearMine,
    asks: (key) => asks(detail, key),
  });
  if (flow.steps.length === 0) return null;
  const shown =
    view.shown && flow.steps.includes(view.shown) ? view.shown : flow.home;
  const index = shown ? flow.steps.indexOf(shown) : -1;
  const done = index < flow.current && shown !== flow.active;

  return (
    <VetoStepsFrame
      steps={stepItems(detail, flow)}
      current={flow.current}
      label={t("label")}
      onSelect={(i) => view.show(flow.steps[i])}
      selected={index}
      extra={detail.veto?.status === "done" && <CopyVetoResult />}
    >
      {shown === "ready" && <VetoReady done={done} />}
      {shown === "maps" && <MapsStep done={done} />}
      {shown === "champions" && <ChampionsStep rule={rule} done={done} />}
      {flow.steps
        .filter((key) => stepGame(key) != null && games[key])
        .map((key) => (
          <div key={key} hidden={shown !== key}>
            {games[key]}
          </div>
        ))}
    </VetoStepsFrame>
  );
}
