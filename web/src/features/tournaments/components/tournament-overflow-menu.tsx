"use client";

import { useTranslations } from "next-intl";

import { MY_PARTICIPATION_ID } from "@/features/participants/lib/focus";
import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import {
  type OverflowAction,
  OverflowActions,
} from "@/shared/ui/overflow-actions";
import { TOURNAMENT_TITLE_ID } from "../lib/focus";

type Props = {
  tenant: string;
  slug: string;
  /** Quem vê fez o check-in, que ainda está aberto: pode desfazer. */
  undoCheckIn: boolean;
  /** Quem vê está inscrito, com a inscrição aberta: pode cancelar. */
  withdrawRegistration: boolean;
  reopenRegistration: boolean;
  /** Em andamento: desfazer o início apaga `results` partidas com mapa ou resultado. */
  undoStart?: { results: number; hasCheckIn: boolean };
  deleteDraft: boolean;
};

/**
 * O menu ⋯ do cabeçalho do torneio. Cancelar a inscrição, reabrir e excluir
 * pedem confirmação; desfazer o início também, e pede o endereço do torneio
 * digitado quando há resultado a perder. Depois, o foco vai para "Sua
 * situação" (o que é de quem vê) ou para o título (o que é do ciclo).
 */
export function TournamentOverflowMenu({
  tenant,
  slug,
  undoCheckIn,
  withdrawRegistration,
  reopenRegistration,
  undoStart,
  deleteDraft,
}: Props) {
  const t = useTranslations("Tournament");
  const tParticipants = useTranslations("Participants");
  const tLifecycle = useTranslations("Lifecycle");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const params = { path: { slug: tenant, tournament: slug } };
  const viewerDone = () => {
    router.refresh();
    focusIfLost(MY_PARTICIPATION_ID);
    focusIfLost(TOURNAMENT_TITLE_ID);
  };
  const lifecycleDone = () => {
    router.refresh();
    focusIfLost(TOURNAMENT_TITLE_ID);
  };

  const items: OverflowAction[] = [];
  if (undoCheckIn) {
    items.push({
      label: tParticipants("checkIn.undo"),
      onSelect: () =>
        run(
          () =>
            api.DELETE("/tenants/{slug}/tournaments/{tournament}/check-in", {
              params,
            }),
          viewerDone,
        ),
    });
  }
  if (withdrawRegistration) {
    items.push({
      label: tParticipants("registration.withdraw"),
      destructive: true,
      confirm: { question: tParticipants("registration.withdrawConfirm") },
      onSelect: () =>
        run(
          () =>
            api.DELETE(
              "/tenants/{slug}/tournaments/{tournament}/registration",
              { params },
            ),
          viewerDone,
        ),
    });
  }
  if (reopenRegistration) {
    items.push({
      label: tLifecycle("reopenRegistration"),
      confirm: { question: tLifecycle("reopenConfirm") },
      onSelect: () =>
        run(
          () =>
            api.POST(
              "/tenants/{slug}/tournaments/{tournament}/registration/reopen",
              { params },
            ),
          lifecycleDone,
        ),
    });
  }
  if (undoStart) {
    const back = tLifecycle(
      undoStart.hasCheckIn ? "undoStartConfirm" : "undoStartConfirmNoCheckIn",
    );
    const lost = tLifecycle("undoStartResults", { count: undoStart.results });
    items.push({
      label: tLifecycle("undoStart"),
      destructive: true,
      confirm: {
        question: `${back} ${lost}`,
        // Com resultado a perder, o forte: digitar o endereço do torneio.
        slug: undoStart.results > 0 ? slug : undefined,
      },
      onSelect: () =>
        run(
          () =>
            api.DELETE("/tenants/{slug}/tournaments/{tournament}/start", {
              params,
            }),
          lifecycleDone,
        ),
    });
  }
  if (deleteDraft) {
    items.push({
      label: t("delete"),
      destructive: true,
      confirm: { question: t("deleteConfirm") },
      onSelect: () =>
        run(
          () =>
            api.DELETE("/tenants/{slug}/tournaments/{tournament}", {
              params,
            }),
          () => {
            router.push(`/${tenant}`);
            router.refresh();
          },
        ),
    });
  }

  return (
    <ActionWithNote error={error}>
      <OverflowActions items={items} pending={pending} />
    </ActionWithNote>
  );
}
