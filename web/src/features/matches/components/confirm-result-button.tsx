"use client";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import { type MatchTarget, matchParams } from "../lib/match-target";

type Props = { target: MatchTarget; label: string };

/** Para a organização: confirmar o placar enviado (no contestado, resolve). */
export function ConfirmResultButton({ target, label }: Props) {
  const router = useRouter();
  const { run, pending, error } = useApiAction();

  return (
    <div className="flex flex-col items-start gap-1">
      <Button
        size="sm"
        disabled={pending}
        onClick={() =>
          run(
            () =>
              api.POST(
                "/tenants/{slug}/tournaments/{tournament}/matches/{match}/confirm",
                { params: matchParams(target) },
              ),
            () => router.refresh(),
          )
        }
      >
        {label}
      </Button>
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
