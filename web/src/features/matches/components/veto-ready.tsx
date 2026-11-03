"use client";

import { useTranslations } from "next-intl";

import { shownNickname } from "@/features/members/lib/nickname";
import { Button } from "@/shared/ui/button";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { IconIndicator } from "@/shared/ui/icon-indicator";
import { Section } from "@/shared/ui/section";
import { useVetoBoard } from "../hooks/use-veto-board";
import { VetoError } from "./veto-error";
import styles from "./veto-ready.module.scss";

type Props = {
  /** Passo já feito, à vista para ver como terminou: só a lista. */
  done?: boolean;
};

/**
 * O passo "Pronto", com o título: quem está pronto, com "Pronto" para quem joga e ainda não
 * clicou e "Começar o veto agora" para a organização. Feito, a lista e quem
 * começou sem esperar, se foi a organização.
 */
export function VetoReady({ done = false }: Props) {
  const t = useTranslations("Match.veto");
  const tSteps = useTranslations("Match.steps");
  const tMember = useTranslations("Member");
  const { veto, mine, names, canManage, pending, actions } = useVetoBoard();
  if (!veto) return null;
  const ready = { a: veto.ready_a, b: veto.ready_b };
  const canReady = !done && mine != null && !ready[mine];
  const canStart = !done && canManage;
  const forcedBy = veto.history.find((e) => e.kind === "begin")?.actor;

  return (
    <Section title={tSteps("ready")}>
      {!done && (
        <p>
          {t(
            veto.team_pick_seconds != null
              ? "waitingReadyTeams"
              : "waitingReady",
          )}
        </p>
      )}
      <ul className={styles.list}>
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
            <Button pending={pending === "ready"} onClick={actions.ready}>
              {t("readyButton")}
            </Button>
          )}
          {canStart && (
            <ConfirmButton
              question={t("forceStartQuestion")}
              onConfirm={actions.start}
              pending={pending === "start"}
              destructive={false}
              kind={mine ? "ghost" : "tertiary"}
            >
              {t("forceStart")}
            </ConfirmButton>
          )}
        </div>
      )}
      {!done && <VetoError />}
    </Section>
  );
}
