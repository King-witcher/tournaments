"use client";

import { useTranslations } from "next-intl";

import { shownNickname } from "@/features/members/lib/nickname";
import { Button } from "@/shared/ui/button";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { ErrorText } from "@/shared/ui/error-text";
import { IconIndicator } from "@/shared/ui/icon-indicator";
import { useLiveMatch } from "../hooks/use-live-match";
import { useSideNames } from "../hooks/use-side-names";
import { useVetoAction } from "../hooks/use-veto-action";
import type { MatchTarget } from "../lib/match-target";
import styles from "./veto-turn.module.scss";

type Props = {
  target: MatchTarget;
  /** Passo já feito, à vista para ver como terminou: só a lista. */
  done?: boolean;
};

/**
 * O passo "Pronto": quem está pronto, com "Pronto" para quem joga e ainda não
 * clicou e "Começar o veto agora" para a organização. Feito, a lista e quem
 * começou sem esperar, se foi a organização.
 */
export function VetoReady({ target, done = false }: Props) {
  const t = useTranslations("Match.veto");
  const tMember = useTranslations("Member");
  const { detail } = useLiveMatch();
  const names = useSideNames(detail);
  const action = useVetoAction(target);
  const veto = detail.veto;
  if (!veto) return null;
  const mine = detail.viewer_side ?? null;
  const ready = { a: veto.ready_a, b: veto.ready_b };
  const canReady = !done && mine != null && !ready[mine];
  const canStart = !done && detail.can_manage;
  const forcedBy = veto.history.find((e) => e.kind === "begin")?.actor;

  return (
    <div className={styles.turn}>
      {!done && (
        <p>
          {t(
            veto.team_pick_seconds != null
              ? "waitingReadyTeams"
              : "waitingReady",
          )}
        </p>
      )}
      <ul className={styles.ready}>
        {(["a", "b"] as const).map((side) => (
          <li key={side}>
            <IconIndicator
              kind={ready[side] ? "succeeded" : "not-started"}
              label={t(ready[side] ? "isReady" : "notReady", {
                name: names[side],
              })}
            />
          </li>
        ))}
      </ul>
      {done && forcedBy != null && (
        <p>
          {t("history.kinds.begin", {
            forced: "yes",
            actor: shownNickname(forcedBy, tMember("deletedAccount")),
          })}
        </p>
      )}
      {(canReady || canStart) && (
        <div className={styles.buttons}>
          {canReady && (
            <Button pending={action.pending === "ready"} onClick={action.ready}>
              {t("readyButton")}
            </Button>
          )}
          {canStart && (
            <ConfirmButton
              question={t("forceStartQuestion")}
              onConfirm={action.start}
              pending={action.pending === "start"}
              destructive={false}
              kind={mine ? "ghost" : "tertiary"}
            >
              {t("forceStart")}
            </ConfirmButton>
          )}
        </div>
      )}
      {action.error && <ErrorText>{action.error}</ErrorText>}
    </div>
  );
}
