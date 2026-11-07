"use client";

import { useTranslations } from "next-intl";
import { useCallback, useRef, useState } from "react";

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
import { NewTournamentModal } from "./new-tournament-modal";
import styles from "./tournament-overflow-menu.module.scss";

type Props = {
  tenant: string;
  slug: string;
  /** Organização: "Copiar torneio" abre a criação com este torneio em "Começar de". */
  copySource?: components["schemas"]["Tournament"];
  /** Quem vê fez o check-in, que ainda está aberto: pode desfazer. */
  undoCheckIn: boolean;
  /** Quem vê está inscrito, com a inscrição aberta: pode cancelar. */
  withdrawRegistration: boolean;
  /** Inscrição só por link: a confirmação de cancelar diz como voltar. */
  inviteOnly: boolean;
  /** Em andamento: desfazer o início apaga `results` partidas com mapa ou resultado. */
  undoStart?: { results: number; hasCheckIn: boolean; several: boolean };
  /** A fase 2 em diante que acabou de começar e ainda não teve jogo: desfazer só o início dela. */
  undoStage?: number;
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
  copySource,
  undoCheckIn,
  withdrawRegistration,
  inviteOnly,
  undoStart,
  undoStage,
  canDelete,
  status,
  countsForRating,
}: Props) {
  const t = useTranslations("Tournament");
  const tParticipants = useTranslations("Participants");
  const tLifecycle = useTranslations("Lifecycle");
  const tStages = useTranslations("Stages");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const [copying, setCopying] = useState(false);
  const stopCopying = useCallback(() => setCopying(false), []);
  // O ⋯, para o foco voltar a ele quando o Modal fecha sem criar.
  const wrapper = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLElement | null>(null);
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
  if (copySource) {
    items.push({
      label: t("copy"),
      onSelect: () => {
        trigger.current = wrapper.current?.querySelector("button") ?? null;
        setCopying(true);
      },
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
  if (undoStage) {
    items.push({
      label: tStages("undoStart", { number: undoStage }),
      destructive: true,
      confirm: { question: tStages("undoStartConfirm", { number: undoStage }) },
      onSelect: () =>
        run(
          () =>
            api.DELETE(
              "/tenants/{slug}/tournaments/{tournament}/stages/{stage}/start",
              { params: { path: { ...params.path, stage: undoStage } } },
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
    // O rótulo curto de sempre ("Desfazer fase 2" ao lado diz qual é qual):
    // o menu do Carbon corta o texto que passa da largura dele.
    items.push({
      label: tLifecycle("undoStart"),
      destructive: true,
      confirm: {
        question: undoStart.several
          ? tStages("undoStartTournamentConfirm")
          : `${back} ${lost}`,
        // Com resultado a perder (ou várias fases), o forte: digitar o
        // endereço do torneio.
        slug: undoStart.results > 0 || undoStart.several ? slug : undefined,
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
    <div ref={wrapper} className={styles.contents}>
      <ActionWithNote error={error}>
        <OverflowActions items={items} pending={pending} />
      </ActionWithNote>
      {copying && copySource && (
        <NewTournamentModal
          tenant={tenant}
          title={t("copy")}
          source={copySource}
          trigger={trigger}
          onClose={stopCopying}
        />
      )}
    </div>
  );
}
