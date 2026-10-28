"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";

type Props = {
  tenant: string;
  tournament: string;
  userId: number;
  nickname: string;
};

/**
 * Para organizer e owner, com o torneio em andamento: marca a desistência
 * do inscrito (pede confirmação). As partidas dele viram W.O.
 */
export function WithdrawParticipantButton({
  tenant,
  tournament,
  userId,
  nickname,
}: Props) {
  const t = useTranslations("Participants");
  const router = useRouter();
  const { run, pending, error } = useApiAction();

  return (
    <div className="flex flex-col items-end">
      <Button
        variant="link"
        disabled={pending}
        className="text-red-600 dark:text-red-400"
        onClick={() => {
          if (!window.confirm(t("withdrawConfirm", { name: nickname }))) return;
          run(
            () =>
              api.POST(
                "/tenants/{slug}/tournaments/{tournament}/participants/{user_id}/withdrawal",
                {
                  params: {
                    path: { slug: tenant, tournament, user_id: userId },
                  },
                },
              ),
            () => router.refresh(),
          );
        }}
      >
        {t("withdrawParticipant")}
      </Button>
      {error && <ErrorText size="xs">{error}</ErrorText>}
    </div>
  );
}
