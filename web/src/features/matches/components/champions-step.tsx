"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import type { components } from "@/shared/api/schema";
import { RadioGroup } from "@/shared/ui/radio-group";
import { useLiveMatch } from "../hooks/use-live-match";
import type { MatchTarget } from "../lib/match-target";
import { currentGame } from "../lib/veto";
import styles from "./champions-step.module.scss";
import { DraftSummary } from "./draft-summary";
import { VetoChampionBoard } from "./veto-champion-board";
import { VetoTurn } from "./veto-turn";
import { VetoUndo } from "./veto-undo";
import { VetoVoteBoard } from "./veto-vote-board";

type Props = {
  target: MatchTarget;
  rule: components["schemas"]["NoRepeatRule"] | null;
  /** Passo feito, à vista para ver como terminou: só o resumo. */
  done?: boolean;
};

/**
 * O passo Campeões: de quem é a vez, desfazer, o resumo do draft (bans e
 * cada partida, com a da vez em destaque, junto da grade) e a grade. Na
 * votação, a do mapa da partida 1 vem antes (o draft depende dele); na
 * janela do TDM, quem vê escolhe para qual partida está escolhendo.
 */
export function ChampionsStep({ target, rule, done = false }: Props) {
  const t = useTranslations("Match.veto.champions");
  const tGame = useTranslations("Match.games");
  const { detail } = useLiveMatch();
  const [chosen, setChosen] = useState<number | null>(null);
  const veto = detail.veto;
  if (!veto) return null;
  if (done) {
    return (
      <div className={styles.alone}>
        <DraftSummary />
      </div>
    );
  }
  const voting = veto.turn?.action === "vote";
  const slots = veto.picks;
  const slot = slots.find((s) => s.game === chosen) ?? slots[0];
  const teamPick = veto.turn?.action === "team_pick";
  const current = voting
    ? undefined
    : (slot?.game ?? veto.turn?.pick_for ?? undefined);

  return (
    <>
      <VetoTurn />
      <VetoUndo target={target} />
      {voting && <VetoVoteBoard target={target} game={currentGame(veto)} />}
      {teamPick && slots.length > 1 && slot && (
        <RadioGroup
          legendText={t("forGame")}
          orientation="horizontal"
          value={String(slot.game)}
          options={slots.map((s) => ({
            value: String(s.game),
            label: tGame("game", { number: s.game }),
          }))}
          onChange={(g) => setChosen(Number(g))}
        />
      )}
      <div className={styles.draft}>
        <div className={styles.summary}>
          <DraftSummary current={current} />
        </div>
        {!voting && (
          <div className={styles.grid}>
            <VetoChampionBoard target={target} rule={rule} slot={slot} />
          </div>
        )}
      </div>
    </>
  );
}
