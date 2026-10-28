"use client";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { ORGANIZER_PANEL_ID } from "../lib/organizer-panel";

/** Desfazer o início tem botão próprio, com confirmação forte (`UndoStartButton`). */
export type Transition = "open_check_in" | "reopen_registration" | "start";

type Props = {
  tenant: string;
  slug: string;
  transition: Transition;
  label: string;
  /** Pergunta antes de chamar a API. */
  confirm?: string;
  variant?: "default" | "outline";
  /** Sem como dar certo agora (quem monta diz por quê ao lado). */
  disabled?: boolean;
};

function call(transition: Transition, tenant: string, slug: string) {
  const params = { path: { slug: tenant, tournament: slug } };
  switch (transition) {
    case "open_check_in":
      return api.POST(
        "/tenants/{slug}/tournaments/{tournament}/check-in/open",
        { params },
      );
    case "reopen_registration":
      return api.POST(
        "/tenants/{slug}/tournaments/{tournament}/registration/reopen",
        { params },
      );
    case "start":
      return api.POST("/tenants/{slug}/tournaments/{tournament}/start", {
        params,
      });
  }
}

/** Para organizer e owner: uma transição de status do torneio. */
export function TransitionButton({
  tenant,
  slug,
  transition,
  label,
  confirm,
  variant = "outline",
  disabled,
}: Props) {
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const start = () =>
    run(
      () => call(transition, tenant, slug),
      () => {
        router.refresh();
        focusIfLost(ORGANIZER_PANEL_ID);
      },
    );

  return (
    <div className="flex flex-col gap-1">
      {confirm ? (
        <ConfirmButton
          size="sm"
          variant={variant}
          pending={pending}
          disabled={disabled}
          danger={false}
          question={confirm}
          onConfirm={start}
        >
          {label}
        </ConfirmButton>
      ) : (
        <Button
          size="sm"
          variant={variant}
          pending={pending}
          disabled={disabled}
          onClick={start}
        >
          {label}
        </Button>
      )}
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
