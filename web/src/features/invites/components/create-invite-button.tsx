"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import { InviteLink } from "./invite-link";

/**
 * Gera um link de convite e mostra uma vez (a API só guarda o hash). O link
 * vai sem idioma: o proxy do next-intl manda cada pessoa para o seu.
 */
export function CreateInviteButton({ slug }: { slug: string }) {
  const t = useTranslations("Tenant.invites");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const [link, setLink] = useState<string>();

  return (
    <div className="flex flex-col items-start gap-2">
      <Button
        pending={pending}
        onClick={() =>
          run(
            () =>
              api.POST("/tenants/{slug}/invites", {
                params: { path: { slug } },
              }),
            (invite) => {
              setLink(`${window.location.origin}/invite/${invite.token}`);
              router.refresh();
            },
          )
        }
      >
        {t("create")}
      </Button>
      {error && <ErrorText>{error}</ErrorText>}
      {/* `key`: um link novo volta o botão de copiar para "copiar". */}
      {link && <InviteLink key={link} link={link} />}
    </div>
  );
}
