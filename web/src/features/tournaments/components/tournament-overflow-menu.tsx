"use client";

import { useTranslations } from "next-intl";

import { MY_PARTICIPATION_ID } from "@/features/participants/lib/focus";
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
import { TOURNAMENT_TITLE_ID } from "../lib/focus";
import { hasStarted } from "../lib/status";

type Props = {
  tenant: string;
  slug: string;
  /** Organização: "Copiar torneio", que abre a criação com `?from=`. */
  copy: boolean;
  /** Quem vê fez o check-in, que ainda está aberto: pode desfazer. */
  undoCheckIn: boolean;
  /** Quem vê está inscrito, com a inscrição aberta: pode cancelar. */
  withdrawRegistration: boolean;
  /** Inscrição só por link: a confirmação de cancelar diz como voltar. */
  inviteOnly: boolean;
  /** Em andamento: desfazer o início apaga `results` partidas com mapa ou resultado. */
  undoStart?: { results: number; hasCheckIn: boolean };
  /** Quem vê pode excluir o torneio, no `status` em que ele está. */
  canDelete: boolean;
  status: components["schemas"]["TournamentStatus"];
  /** Excluir um torneio começado que conta para o rating refaz o ranking. */
  countsForRating: boolean;
};

/**
 * O menu ⋯ do cabeçalho do torneio. Cancelar a inscrição e excluir
 * pedem confirmação, e excluir o que já foi publicado pede o endereço digitado;
 * desfazer o início também, quando há resultado a perder. Depois, o foco vai
 * para "Sua situação" (o que é de quem vê) ou para o título (o que é do ciclo).
 */
export function TournamentOverflowMenu({
  tenant,
  slug,
  copy,
  undoCheckIn,
  withdrawRegistration,
  inviteOnly,
  undoStart,
  canDelete,
  status,
  countsForRating,
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
  if (copy) {
    items.push({
      label: t("copy"),
      href: { pathname: `/${tenant}/new`, query: { from: slug } },
    });
  }
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
      confirm: {
        question: tParticipants(
          inviteOnly
            ? "registration.withdrawConfirmInviteOnly"
            : "registration.withdrawConfirm",
        ),
      },
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
  if (canDelete) {
    const draft = status === "draft";
    const started = hasStarted(status);
    const lost = t(
      draft
        ? "deleteConfirm"
        : started
          ? "deleteStartedConfirm"
          : "deletePublishedConfirm",
    );
    items.push({
      label: t(draft ? "delete" : "deleteTournament"),
      destructive: true,
      confirm: {
        question:
          started && countsForRating
            ? `${lost} ${t("deleteRatingNote")}`
            : lost,
        // Fora do rascunho, o forte: há gente esperando por ele.
        slug: draft ? undefined : slug,
      },
      onSelect: () =>
        run(
          () =>
            api.DELETE("/tenants/{slug}/tournaments/{tournament}", {
              params,
              body: draft ? undefined : { confirm_slug: slug },
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
