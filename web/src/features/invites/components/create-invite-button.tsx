"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { Button } from "@/shared/ui/button";
import { useReportCreatedInvite } from "./created-invite-provider";

/**
 * Gera um link de convite; o link (ou o erro) aparece acima da tabela, pelo
 * `CreatedInviteProvider`. O link vai sem idioma: o proxy do next-intl manda
 * cada pessoa para o seu.
 */
export function CreateInviteButton({ slug }: { slug: string }) {
  const t = useTranslations("Tenant.invites");
  const router = useRouter();
  const report = useReportCreatedInvite();
  const errorMessage = useApiErrorMessage();
  const { run, pending } = useApiAction();

  return (
    <Button
      pending={pending}
      onClick={() =>
        run(
          () =>
            api.POST("/tenants/{slug}/invites", {
              params: { path: { slug } },
            }),
          (invite) => {
            report({
              link: `${window.location.origin}/invite/${invite.token}`,
            });
            router.refresh();
          },
          (code) => report({ error: errorMessage(code) }),
        )
      }
    >
      {t("create")}
    </Button>
  );
}
