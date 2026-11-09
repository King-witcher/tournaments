"use client";

import { useTranslations } from "next-intl";
import { shownNickname } from "@/features/members/lib/nickname";
import { Button } from "@/shared/ui/button";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { IconIndicator } from "@/shared/ui/icon-indicator";
import { Panel } from "@/shared/ui/panel";
import { Section } from "@/shared/ui/section";
import { useVetoBoard } from "../hooks/use-veto-board";
import { VetoError } from "./veto-error";
import styles from "./veto-ready.module.scss";

type Props = {
  /** Passo já feito, à vista para ver como terminou: só a lista. */
  done?: boolean;
  /** Na sala de veto: "Começar agora" é da gestão, e os textos não falam em organização. */
  room?: boolean;
};

/**
 * O passo "Pronto", com o título: cada lado, com quem falta por nome, "Pronto"
 * para quem ainda precisa marcar, "Não estou mais pronto" para quem marcou e
 * cansou de esperar, o aviso de que é o capitão quem marca para o
 * colega e "Começar o veto agora" para a organização (na sala, para a gestão).
 * Feito, a lista e quem começou sem esperar, se foi a organização.
 */
export function VetoReady({ done = false, room = false }: Props) {
  const t = useTranslations("Match.veto");
  const tSteps = useTranslations("Match.steps");
  const tMember = useTranslations("Member");
  const { veto, mine, names, canManage, pending, actions } = useVetoBoard();
  if (!veto) return null;
  const ready = { a: veto.ready_a, b: veto.ready_b };
  const canReady = !done && veto.can_act.ready;
  const canUnready = !done && veto.can_act.unready;
  const canStart = !done && canManage;
  const forcedBy = veto.history.find((e) => e.kind === "begin")?.actor;
  const teams = veto.team_size > 1;
  const byCaptain = veto.ready_by === "captain";
  // O colega que não marca: o lado dele ainda espera o capitão.
  const waitsCaptain =
    !done && byCaptain && mine != null && !ready[mine] && !canReady;
  const waiting = (side: "a" | "b") =>
    veto.ready_waiting
      .filter((p) => p.side === side)
      .map((p) => shownNickname(p.nickname, tMember("deletedAccount")))
      .join(", ");

  return (
    <Panel>
      <Section title={tSteps("ready")}>
        {!done && (
          <p>
            {t(
              !teams
                ? "waitingReady"
                : byCaptain
                  ? "waitingReadyCaptains"
                  : "waitingReadyTeams",
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
              {!done &&
                teams &&
                !veto.nameless &&
                !ready[side] &&
                waiting(side) && (
                  <p className={styles.waiting}>
                    {t("waitingFor", { names: waiting(side) })}
                  </p>
                )}
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
        {waitsCaptain && <p>{t("captainActs.ready")}</p>}
        {(canReady || canUnready || canStart) && (
          <div className={styles.buttons}>
            {canReady && (
              <Button pending={pending === "ready"} onClick={actions.ready}>
                {t("readyButton")}
              </Button>
            )}
            {canUnready && (
              <Button
                kind="secondary"
                pending={pending === "unready"}
                onClick={actions.unready}
              >
                {t("unreadyButton")}
              </Button>
            )}
            {canStart && (
              <ConfirmButton
                question={t(
                  room ? "forceStartRoomQuestion" : "forceStartQuestion",
                )}
                onConfirm={actions.start}
                pending={pending === "start"}
                destructive={false}
                kind={mine ? "ghost" : "tertiary"}
              >
                {t(room ? "forceStartNow" : "forceStart")}
              </ConfirmButton>
            )}
          </div>
        )}
        {!done && <VetoError />}
      </Section>
    </Panel>
  );
}
