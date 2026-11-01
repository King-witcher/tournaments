"use client";

import { useTranslations } from "next-intl";
import { type ReactNode, useEffect, useRef, useState } from "react";

import type { components } from "@/shared/api/schema";
import { focusIfLost } from "@/shared/lib/focus";
import { ProgressSteps } from "@/shared/ui/progress-steps";
import { useLiveMatch } from "../hooks/use-live-match";
import { useStepItems } from "../hooks/use-step-items";
import { type StepKey, seriesSteps, stepGame } from "../lib/match-steps";
import { type MatchTarget, VETO_ID } from "../lib/match-target";
import { ChampionsStep } from "./champions-step";
import { MapsStep } from "./maps-step";
import styles from "./match-steps.module.scss";
import { VetoReady } from "./veto-ready";

type Props = {
  target: MatchTarget;
  rule: components["schemas"]["NoRepeatRule"] | null;
  /** Cada partida que abre (as enviadas e a da vez), desenhada no servidor. */
  games: Partial<Record<StepKey, ReactNode>>;
};

/**
 * Os passos da série (Pronto, Mapas, Campeões e cada partida; só os que o
 * torneio tem) e o passo à vista, um por vez. Abre no que pede uma ação de
 * quem vê, senão no atual, e segue para o novo quando ele muda; o indicador
 * é a navegação: os passos feitos e o atual abrem, os seguintes não. As
 * partidas ficam montadas mesmo escondidas (o formulário de envio não perde
 * o que já foi preenchido). Com um passo só, sem indicador.
 */
export function MatchSteps({ target, rule, games }: Props) {
  const t = useTranslations("Match.steps");
  const stepItems = useStepItems();
  const { detail } = useLiveMatch();
  const flow = seriesSteps(detail);
  const [viewing, setViewing] = useState<StepKey | null>(null);
  const [seen, setSeen] = useState(flow.home);
  if (flow.home !== seen) {
    setSeen(flow.home);
    setViewing(null);
  }
  // O passo mudou sozinho (depois de agir, o botão usado some): o foco vai
  // para o passo novo. Não na primeira vez, que é a página abrindo.
  const focused = useRef(seen);
  useEffect(() => {
    if (focused.current === seen) return;
    focused.current = seen;
    focusIfLost(VETO_ID);
  });
  if (flow.steps.length === 0) return null;
  const shown = viewing && flow.steps.includes(viewing) ? viewing : flow.home;
  const index = flow.steps.indexOf(shown);
  const done = index < flow.current;

  return (
    <div className={styles.steps}>
      {flow.steps.length > 1 && (
        <ProgressSteps
          steps={stepItems(detail, flow)}
          current={flow.current}
          label={t("label")}
          onSelect={(i) => setViewing(flow.steps[i])}
          selected={index}
        />
      )}
      {/* tabIndex -1: recebe o foco depois de um passo cujo botão some. */}
      <div id={VETO_ID} tabIndex={-1} className={styles.panel}>
        {shown === "ready" && <VetoReady target={target} done={done} />}
        {shown === "maps" && <MapsStep target={target} done={done} />}
        {shown === "champions" && (
          <ChampionsStep target={target} rule={rule} done={done} />
        )}
        {flow.steps
          .filter((key) => stepGame(key) != null && games[key])
          .map((key) => (
            <div key={key} hidden={shown !== key}>
              {games[key]}
            </div>
          ))}
      </div>
    </div>
  );
}
