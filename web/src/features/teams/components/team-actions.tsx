"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import {
  type OverflowAction,
  OverflowActions,
} from "@/shared/ui/overflow-actions";
import { useMenuTrigger } from "../hooks/use-menu-trigger";
import { useTeamName } from "../hooks/use-team-name";
import { teamBlockId } from "../lib/focus";
import { CaptainModal } from "./captain-modal";
import { RenameTeamModal } from "./rename-team-modal";
import { type Candidate, SubstitutionModal } from "./substitution-modal";

type Props = {
  tenant: string;
  tournament: string;
  team: components["schemas"]["Team"];
  /** Renomear e trocar o capitão: até o torneio encerrar. */
  editable: boolean;
  /** Substituir e marcar desistência: em andamento, time da chave que não desistiu. */
  inPlay: boolean;
  /** Para a substituição: os reservas ativos e os membros não inscritos. */
  reserves: Candidate[];
  others: Candidate[];
};

type Dialog = "rename" | "captain" | "substitute";

/**
 * O ⋯ de um time na aba Inscritos, para a organização: renomear, trocar o
 * capitão, substituir um jogador e marcar a desistência do time (com a
 * pergunta). Depois, a página recarrega e o foco volta ao menu (ou ao bloco
 * do time, se o menu sumiu).
 */
export function TeamActions({
  tenant,
  tournament,
  team,
  editable,
  inPlay,
  reserves,
  others,
}: Props) {
  const t = useTranslations("Team");
  const name = useTeamName()(team);
  const router = useRouter();
  const { wrapper, trigger } = useMenuTrigger();
  const [dialog, setDialog] = useState<Dialog>();
  const withdraw = useApiAction();

  const done = () => {
    setDialog(undefined);
    router.refresh();
    requestAnimationFrame(() => trigger.current?.focus());
    focusIfLost(teamBlockId(team.id));
  };
  const close = () => setDialog(undefined);
  const params = { path: { slug: tenant, tournament, team: team.id } };

  const items: OverflowAction[] = [];
  if (editable) {
    items.push({
      label: t("rename.open"),
      onSelect: () => setDialog("rename"),
    });
    if (team.members.filter((m) => !m.left).length > 1) {
      items.push({
        label: t("captain.open"),
        onSelect: () => setDialog("captain"),
      });
    }
  }
  if (inPlay) {
    items.push({
      label: t("substitute.open"),
      onSelect: () => setDialog("substitute"),
    });
    items.push({
      label: t("withdraw.open"),
      destructive: true,
      confirm: { question: t("withdraw.confirm", { name }) },
      onSelect: () =>
        withdraw.run(
          () =>
            api.POST(
              "/tenants/{slug}/tournaments/{tournament}/teams/{team}/withdrawal",
              { params },
            ),
          done,
        ),
    });
  }
  if (items.length === 0) return null;

  const modal = { tenant, tournament, team, returnFocusTo: trigger };
  return (
    <div ref={wrapper}>
      <ActionWithNote align="end" error={withdraw.error}>
        <OverflowActions
          items={items}
          pending={withdraw.pending}
          size="sm"
          label={t("teamActions", { name })}
        />
      </ActionWithNote>
      {dialog === "rename" && (
        <RenameTeamModal {...modal} onClose={close} onDone={done} />
      )}
      {dialog === "captain" && (
        <CaptainModal {...modal} onClose={close} onDone={done} />
      )}
      {dialog === "substitute" && (
        <SubstitutionModal
          {...modal}
          reserves={reserves}
          others={others}
          onClose={close}
          onDone={done}
        />
      )}
    </div>
  );
}
