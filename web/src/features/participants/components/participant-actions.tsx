"use client";

import { useTranslations } from "next-intl";
import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import {
  type OverflowAction,
  OverflowActions,
} from "@/shared/ui/overflow-actions";
import { PARTICIPANTS_ID } from "../lib/focus";

type Props = {
  tenant: string;
  tournament: string;
  userId: number;
  nickname: string;
  /** Durante o check-in: marcar (sem pergunta) ou desmarcar (com pergunta). */
  checkIn?: "mark" | "unmark";
  /** Até começar: tirar do torneio. */
  remove: boolean;
  /** Em andamento, para quem está na chave: marcar a desistência. */
  withdraw: boolean;
};

/**
 * O ⋯ de um inscrito, para organizer e owner. Desmarcar o check-in, tirar e
 * marcar a desistência pedem confirmação (dizem o que acontece). Depois, se
 * a linha sumiu, o foco vai para a tabela.
 */
export function ParticipantActions({
  tenant,
  tournament,
  userId,
  nickname,
  checkIn,
  remove,
  withdraw,
}: Props) {
  const t = useTranslations("Participants");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const path = { slug: tenant, tournament, user_id: userId };
  const checkInPath =
    "/tenants/{slug}/tournaments/{tournament}/participants/{user_id}/check-in";
  const done = () => {
    router.refresh();
    focusIfLost(PARTICIPANTS_ID);
  };

  const items: OverflowAction[] = [];
  if (checkIn === "mark") {
    items.push({
      label: t("checkIn.mark"),
      onSelect: () =>
        run(() => api.PUT(checkInPath, { params: { path } }), done),
    });
  }
  if (checkIn === "unmark") {
    items.push({
      label: t("checkIn.unmark"),
      destructive: true,
      confirm: { question: t("checkIn.unmarkConfirm", { name: nickname }) },
      onSelect: () =>
        run(() => api.DELETE(checkInPath, { params: { path } }), done),
    });
  }
  if (remove) {
    items.push({
      label: t("remove"),
      destructive: true,
      confirm: { question: t("removeConfirm", { name: nickname }) },
      onSelect: () =>
        run(
          () =>
            api.DELETE(
              "/tenants/{slug}/tournaments/{tournament}/participants/{user_id}",
              { params: { path } },
            ),
          done,
        ),
    });
  }
  if (withdraw) {
    items.push({
      label: t("withdrawParticipant"),
      destructive: true,
      confirm: { question: t("withdrawConfirm", { name: nickname }) },
      onSelect: () =>
        run(
          () =>
            api.POST(
              "/tenants/{slug}/tournaments/{tournament}/participants/{user_id}/withdrawal",
              { params: { path } },
            ),
          done,
        ),
    });
  }

  return (
    <ActionWithNote align="end" error={error}>
      <OverflowActions
        items={items}
        pending={pending}
        size="sm"
        label={t("actionsFor", { name: nickname })}
      />
    </ActionWithNote>
  );
}
