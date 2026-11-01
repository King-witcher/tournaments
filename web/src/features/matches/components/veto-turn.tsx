"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { Countdown } from "@/shared/ui/countdown";
import { useLiveMatch } from "../hooks/use-live-match";
import { useSideNames } from "../hooks/use-side-names";
import { currentGame, turnKey } from "../lib/veto";
import styles from "./veto-turn.module.scss";

/**
 * De quem é a vez no veto e o tempo que falta, pela hora da API (no fim, a
 * escolha é sorteada: a página recarrega para mostrar o sorteio); na janela
 * simultânea do TDM, o tempo da janela. Some sem vez (o passo atual mostra o
 * resto) e no instante entre um passo marcado na hora e a resposta quando a
 * vez seguinte é de outra fase (`sides` vazio).
 */
export function VetoTurn() {
  const t = useTranslations("Match.veto");
  const router = useRouter();
  const { detail } = useLiveMatch();
  const names = useSideNames(detail);
  const veto = detail.veto;
  const turn = veto?.turn;
  // Sem ninguém na vez: o passo marcado na hora trocou de fase, e a vez de
  // verdade vem com a resposta.
  if (!veto || !turn || turn.sides.length === 0) return null;
  if (veto.status !== "in_progress") return null;
  const mine = detail.viewer_side ?? null;
  const game = currentGame(veto);

  const deadline = veto.deadline && (
    <Countdown
      until={veto.deadline}
      serverNow={detail.now}
      // O sorteio é feito pela API, que ainda aceita o clique por 2 s depois
      // do prazo: recarrega depois disso para mostrar.
      onExpire={() => setTimeout(() => router.refresh(), 2500)}
    />
  );
  if (turn.action === "team_pick") {
    const yours = veto.picks.length > 0;
    return (
      <div className={styles.turn}>
        <p className={yours ? styles.yours : undefined}>
          {t(yours ? "teamPickYours" : "teamPick")}
        </p>
        {deadline && (
          <p className={styles.note}>
            {t.rich("teamDeadline", { time: () => deadline })}
          </p>
        )}
      </div>
    );
  }
  const what = t(`action.${turnKey(turn)}`, {
    game: turn.pick_for ?? game,
  });
  const ours = mine != null && turn.sides.includes(mine);
  let sentence: string;
  if (turn.sides.length === 2) {
    sentence = t("bothVote", { game });
  } else if (ours) {
    sentence = t("yourTurn", { action: what });
  } else if (turn.action === "vote" && mine) {
    sentence = t("votedWaiting", { name: names[turn.sides[0]] });
  } else {
    sentence = t("theirTurn", { name: names[turn.sides[0]], action: what });
  }
  return (
    <div className={styles.turn}>
      <p className={ours ? styles.yours : undefined}>{sentence}</p>
      {deadline && (
        <p className={styles.note}>
          {t.rich("deadline", { time: () => deadline })}
        </p>
      )}
    </div>
  );
}
