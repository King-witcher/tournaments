"use client";

import { TOURNAMENT_TITLE_ID } from "@/features/tournaments/lib/focus";
import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button } from "@/shared/ui/button";
import { ConfirmButton } from "@/shared/ui/confirm-button";

/** Desfazer o início fica no ⋯ do cabeçalho. */
export type Transition = "open_check_in" | "start";

type Props = {
  tenant: string;
  slug: string;
  transition: Transition;
  label: string;
  /** Pergunta antes de chamar a API (o Modal com o mesmo texto do botão). */
  confirm?: string;
  /** Sem como dar certo agora: o botão fica desabilitado, com este motivo. */
  disabledReason?: string;
  /**
   * O que vai impedir a ação, sem travar o botão (a API confere e responde
   * com o código): junto dele, como o motivo.
   */
  note?: string;
};

function call(transition: Transition, tenant: string, slug: string) {
  const params = { path: { slug: tenant, tournament: slug } };
  switch (transition) {
    case "open_check_in":
      return api.POST(
        "/tenants/{slug}/tournaments/{tournament}/check-in/open",
        { params },
      );
    case "start":
      return api.POST("/tenants/{slug}/tournaments/{tournament}/start", {
        params,
      });
  }
}

/**
 * Para organizer e owner: uma transição do ciclo do torneio (abrir o
 * check-in, iniciar). Depois, o foco vai para o título do torneio, porque o
 * botão some quando o estado muda.
 */
export function TransitionButton({
  tenant,
  slug,
  transition,
  label,
  confirm,
  disabledReason,
  note,
}: Props) {
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const disabled = disabledReason !== undefined;
  const go = () =>
    run(
      () => call(transition, tenant, slug),
      () => {
        router.refresh();
        focusIfLost(TOURNAMENT_TITLE_ID);
      },
    );

  return (
    <ActionWithNote
      note={disabledReason ?? note}
      error={error}
      compact
      blocked={disabled}
    >
      {confirm ? (
        <ConfirmButton
          destructive={false}
          kind="primary"
          size="md"
          pending={pending}
          disabled={disabled}
          question={confirm}
          onConfirm={go}
        >
          {label}
        </ConfirmButton>
      ) : (
        <Button
          kind="primary"
          pending={pending}
          disabled={disabled}
          onClick={go}
        >
          {label}
        </Button>
      )}
    </ActionWithNote>
  );
}
