"use client";

import { useTranslations } from "next-intl";
import { type ReactNode, useState } from "react";

import type { components } from "@/shared/api/schema";
import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
import { ProgressSteps } from "@/shared/ui/progress-steps";
import { useLiveMatch } from "../hooks/use-live-match";
import { type StepKey, seriesSteps } from "../lib/match-steps";
import { type MatchTarget, VETO_ID } from "../lib/match-target";
import { ChampionsStep } from "./champions-step";
import { MapsStep } from "./maps-step";
import styles from "./match-steps.module.scss";
import { VetoError } from "./veto-error";
import { VetoReady } from "./veto-ready";

type Props = {
  target: MatchTarget;
  rule: components["schemas"]["NoRepeatRule"] | null;
  /** O passo Partidas, desenhado no servidor (as partidas e o envio). */
  games: ReactNode;
};

/**
 * Os passos da série abaixo do placar (Pronto, Mapas, Campeões, Partidas;
 * só os que o torneio tem) e o passo à vista: abre no atual e segue para o
 * novo quando ele muda; um passo feito, clicado no indicador, mostra como
 * terminou, com a volta para o atual. As Partidas ficam montadas mesmo
 * escondidas (o formulário de envio não perde o que já foi preenchido).
 * Sem veto nenhum, só as Partidas.
 */
export function MatchSteps({ target, rule, games }: Props) {
  const t = useTranslations("Match.steps");
  const { detail } = useLiveMatch();
  const flow = seriesSteps(detail);
  const home = flow?.home ?? "games";
  const [viewing, setViewing] = useState<StepKey | null>(null);
  const [seen, setSeen] = useState(home);
  if (home !== seen) {
    setSeen(home);
    setViewing(null);
  }
  if (!flow) return games;
  const shown = viewing ?? home;
  const back = shown !== home;
  const finished = flow.current >= flow.steps.length;

  return (
    <div className={styles.steps}>
      <ProgressSteps
        steps={flow.steps.map((key) => ({ key, label: t(key) }))}
        current={flow.current}
        label={t("label")}
        onSelect={(i) => setViewing(flow.steps[i])}
        selected={back || finished ? flow.steps.indexOf(shown) : undefined}
      />
      {/* tabIndex -1: recebe o foco depois de um passo cujo botão some. */}
      <div id={VETO_ID} tabIndex={-1} className={styles.panel}>
        <h2 className={styles.srOnly}>{t(shown)}</h2>
        {back && (
          <div>
            <Button
              kind="tertiary"
              size="sm"
              onClick={() => {
                setViewing(null);
                focusIfLost(VETO_ID);
              }}
            >
              {t("back", { step: t(home) })}
            </Button>
          </div>
        )}
        {shown === "ready" && <VetoReady target={target} done={back} />}
        {shown === "maps" && <MapsStep target={target} done={back} />}
        {shown === "champions" && (
          <ChampionsStep target={target} rule={rule} done={back} />
        )}
        {shown !== "games" && !back && <VetoError />}
        <div hidden={shown !== "games"}>{games}</div>
      </div>
    </div>
  );
}
