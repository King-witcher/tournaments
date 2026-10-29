"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import type { components } from "@/shared/api/schema";
import { Button } from "@/shared/ui/button";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { Countdown } from "@/shared/ui/countdown";
import { ErrorText } from "@/shared/ui/error-text";
import { useVetoAction } from "../hooks/use-veto-action";
import type { MatchTarget } from "../lib/match-target";
import { currentGame, turnKey } from "../lib/veto";
import styles from "./veto-turn.module.scss";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  veto: Schemas["VetoView"];
  names: { a: string; b: string };
  /** O lado de quem vê, se joga. */
  mine: Schemas["Side"] | null;
  canManage: boolean;
};

/**
 * O que o veto espera agora, numa frase: antes de começar, quem está pronto
 * (com "Pronto" para quem joga e "Começar agora" para a organização); depois,
 * de quem é a vez e o tempo que falta (no fim, a escolha é sorteada: a
 * página recarrega para mostrar o sorteio); concluído, que o mapa pode ser
 * enviado.
 */
export function VetoTurn({ target, veto, names, mine, canManage }: Props) {
  const t = useTranslations("Match.veto");
  const router = useRouter();
  const action = useVetoAction(target);
  const game = currentGame(veto);

  if (veto.status === "waiting_ready") {
    const ready = { a: veto.ready_a, b: veto.ready_b };
    const other = mine === "a" ? "b" : "a";
    return (
      <div className={styles.turn}>
        <p>{t("waitingReady")}</p>
        <ul className={styles.ready}>
          {(["a", "b"] as const).map((side) => (
            <li key={side}>
              {t(ready[side] ? "isReady" : "notReady", { name: names[side] })}
            </li>
          ))}
        </ul>
        <div className={styles.buttons}>
          {mine && !ready[mine] && (
            <Button pending={action.pending} onClick={action.ready}>
              {t("readyButton")}
            </Button>
          )}
          {mine && ready[mine] && (
            <p className={styles.note}>
              {t("youAreReady", { name: names[other] })}
            </p>
          )}
          {canManage && (
            <ConfirmButton
              question={t("forceStartQuestion")}
              onConfirm={action.start}
              pending={action.pending}
              destructive={false}
              kind={mine ? "ghost" : "tertiary"}
            >
              {t("forceStart")}
            </ConfirmButton>
          )}
        </div>
        {action.error && <ErrorText>{action.error}</ErrorText>}
      </div>
    );
  }

  if (veto.status === "done" || !veto.turn) {
    return <p className={styles.turn}>{t("done", { game })}</p>;
  }

  const turn = veto.turn;
  const what = t(`action.${turnKey(turn)}`, { game });
  const mineToAct = mine != null && turn.sides.includes(mine);
  let sentence: string;
  if (turn.sides.length === 2) {
    sentence = t("bothVote", { game });
  } else if (mineToAct) {
    sentence = t("yourTurn", { action: what });
  } else if (turn.action === "vote" && mine) {
    sentence = t("votedWaiting", { name: names[turn.sides[0]] });
  } else {
    sentence = t("theirTurn", { name: names[turn.sides[0]], action: what });
  }
  return (
    <div className={styles.turn}>
      <p className={mineToAct ? styles.yours : undefined}>{sentence}</p>
      {veto.deadline && (
        <p className={styles.note}>
          {t.rich("deadline", {
            time: () => (
              <Countdown
                until={veto.deadline ?? ""}
                // O sorteio é feito pela API: recarrega para mostrar.
                onExpire={() => setTimeout(() => router.refresh(), 1000)}
              />
            ),
          })}
        </p>
      )}
    </div>
  );
}
