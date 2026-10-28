"use client";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";

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
}: Props) {
  const router = useRouter();
  const { run, pending, error } = useApiAction();

  return (
    <div className="flex flex-col gap-1">
      <Button
        size="sm"
        variant={variant}
        disabled={pending}
        onClick={() => {
          if (confirm && !window.confirm(confirm)) return;
          run(
            () => call(transition, tenant, slug),
            () => router.refresh(),
          );
        }}
      >
        {label}
      </Button>
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
