"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import { InviteLink } from "./invite-link";

type Props = { tenant: string; tournament: string };

/**
 * Gera um link de inscrição no torneio e mostra uma vez (a API só guarda o
 * hash). É o mesmo `/invite/{token}` do convite do tenant.
 */
export function CreateTournamentInviteButton({ tenant, tournament }: Props) {
  const t = useTranslations("Tournament.invites");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const [link, setLink] = useState<string>();

  return (
    <div className="flex flex-col items-start gap-2">
      <Button
        variant="outline"
        size="sm"
        disabled={pending}
        onClick={() =>
          run(
            () =>
              api.POST("/tenants/{slug}/tournaments/{tournament}/invites", {
                params: { path: { slug: tenant, tournament } },
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
