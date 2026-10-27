"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";

/** Aceita o convite e leva para a página da comunidade. */
export function AcceptInviteButton({ token }: { token: string }) {
  const t = useTranslations("Invite");
  const router = useRouter();
  const { run, pending, error } = useApiAction();

  return (
    <div className="flex flex-col items-start gap-2">
      <Button
        disabled={pending}
        onClick={() =>
          run(
            () =>
              api.POST("/invites/{token}/accept", {
                params: { path: { token } },
              }),
            (tenant) => router.push(`/${tenant.slug}`),
          )
        }
      >
        {t("accept")}
      </Button>
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
