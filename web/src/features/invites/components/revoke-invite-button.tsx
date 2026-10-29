"use client";

import { Stack } from "@carbon/react";
import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { ErrorText } from "@/shared/ui/error-text";

/**
 * Revoga um link (de convite ou de inscrição: a rota é a mesma), depois da
 * confirmação. Na tabela de convites da comunidade a ação fica no ⋯
 * (`InviteActions`); este botão serve a quem lista links fora de tabela.
 */
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
    <Stack gap={2}>
      <ConfirmButton
        pending={pending}
        question={t("revokeConfirm", { has: "no", link: "" })}
        onConfirm={() =>
          run(
            () =>
              api.DELETE("/tenants/{slug}/invites/{invite_id}", {
                params: { path: { slug, invite_id: inviteId } },
              }),
            () => router.refresh(),
          )
        }
      >
        {t("revoke")}
      </ConfirmButton>
      {error && <ErrorText>{error}</ErrorText>}
    </Stack>
  );
}
