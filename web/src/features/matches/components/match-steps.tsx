"use client";

import { useTranslations } from "next-intl";
import { type ReactNode, useEffect, useState } from "react";

import type { components } from "@/shared/api/schema";
import { focusIfLost } from "@/shared/lib/focus";
import { ProgressSteps } from "@/shared/ui/progress-steps";
import { useLiveMatch } from "../hooks/use-live-match";
import { useStepItems } from "../hooks/use-step-items";
import { asks, type StepKey, seriesSteps, stepGame } from "../lib/match-steps";
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

/** O foco está num controle do passo à vista (alguém lendo ou digitando nele). */
const focusInside = () => {
  const panel = document.getElementById(VETO_ID);
  const active = document.activeElement;
  return panel != null && active !== panel && panel.contains(active);
};

/**
 * Os passos da série (Pronto, Mapas, Campeões e cada partida; só os que o
 * torneio tem) e o passo à vista, um por vez. Abre no que pede uma ação de
 * quem vê, senão no atual; o indicador é a navegação (os passos feitos e o
 * atual abrem, os seguintes não). Quando o passo que abre muda, a tela só
 * segue sozinha se a mudança veio de uma ação de quem vê (e então o foco
 * vai junto) ou se ele estava nesse passo, que não pede mais nada dele, sem
 * o foco num controle dele; com uma ação do veto no ar, espera a resposta.
 * Nos outros casos fica onde está: o indicador mostra o estado do passo
 * novo. As partidas ficam montadas mesmo escondidas (o formulário de envio
 * não perde o que já foi preenchido). Com um passo só, sem indicador.
 */
export function MatchSteps({ target, rule, games }: Props) {
  const t = useTranslations("Match.steps");
  const stepItems = useStepItems();
  const { detail, pending, isMine, clearMine } = useLiveMatch();
  const flow = seriesSteps(detail);
  const [view, setView] = useState({ home: flow.home, shown: flow.home });
  const [focusTurn, setFocusTurn] = useState(0);
  if (flow.home !== view.home && pending == null) {
    const mine = isMine();
    const follow =
      mine ||
      (view.shown === view.home && !asks(detail, view.home) && !focusInside());
    setView({ home: flow.home, shown: follow ? flow.home : view.shown });
    if (mine) setFocusTurn((n) => n + 1);
  }
  // A mudança da ação de quem vê: o botão usado sumiu, o foco vai para o
  // passo novo.
  useEffect(() => {
    if (focusTurn === 0) return;
    clearMine();
    focusIfLost(VETO_ID);
    // O foco da ação do veto já pode estar no passo, sem ter rolado: o passo
    // novo começa do topo, se o topo ficou para cima.
    const panel = document.getElementById(VETO_ID);
    if (!panel) return;
    const margin = Number.parseFloat(
      getComputedStyle(panel).scrollMarginBlockStart,
    );
    if (panel.getBoundingClientRect().top < margin) panel.scrollIntoView();
  }, [focusTurn, clearMine]);
  if (flow.steps.length === 0) return null;
  const shown =
    view.shown && flow.steps.includes(view.shown) ? view.shown : flow.home;
  const index = shown ? flow.steps.indexOf(shown) : -1;
  const done = index < flow.current && shown !== flow.active;

  return (
    <div className={styles.steps}>
      {flow.steps.length > 1 && (
        <ProgressSteps
          steps={stepItems(detail, flow)}
          current={flow.current}
          label={t("label")}
          onSelect={(i) => setView({ ...view, shown: flow.steps[i] })}
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
