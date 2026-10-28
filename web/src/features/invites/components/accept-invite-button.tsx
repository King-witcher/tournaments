"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";

type Props = {
  token: string;
  /** Link de inscrição: aceitar também inscreve no torneio. */
  tournament: boolean;
};

/** Aceita o convite e leva para a página da comunidade ou do torneio. */
export function AcceptInviteButton({ token, tournament }: Props) {
  const t = useTranslations("Invite");
  const router = useRouter();
  const { run, pending, error } = useApiAction();

  return (
    <div className="flex flex-col items-start gap-2">
      <Button
        pending={pending}
        onClick={() =>
          run(
            () =>
              api.POST("/invites/{token}/accept", {
                params: { path: { token } },
              }),
            (accepted) =>
              router.push(
                accepted.tournament
                  ? `/${accepted.tenant.slug}/${accepted.tournament}`
                  : `/${accepted.tenant.slug}`,
              ),
          )
        }
      >
        {tournament ? t("acceptTournament") : t("accept")}
      </Button>
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
