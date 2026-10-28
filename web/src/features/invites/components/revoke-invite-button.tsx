"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { ConfirmButton } from "@/shared/ui/confirm-button";

export function RevokeInviteButton({
  slug,
  inviteId,
}: {
  slug: string;
  inviteId: number;
}) {
  const t = useTranslations("Tenant.invites");
  const router = useRouter();
  const { run, pending, error } = useApiAction();

  return (
    <div className="flex flex-col items-end">
      <ConfirmButton
        variant="link"
        pending={pending}
        question={t("revokeConfirm")}
        onConfirm={() =>
          run(
            () =>
              api.DELETE("/tenants/{slug}/invites/{invite_id}", {
                params: { path: { slug, invite_id: inviteId } },
              }),
            () => router.refresh(),
          )
        }
        className="text-red-600 dark:text-red-400"
      >
        {t("revoke")}
      </ConfirmButton>
      {error && <ErrorText size="xs">{error}</ErrorText>}
    </div>
  );
}
