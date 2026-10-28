"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { ConfirmButton } from "@/shared/ui/confirm-button";

type Props = {
  tenant: string;
  tournament: string;
  userId: number;
  nickname: string;
};

/** Para organizer e owner: tira o inscrito do torneio, depois de confirmar. */
export function RemoveParticipantButton({
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
      <ConfirmButton
        variant="link"
        pending={pending}
        question={t("removeConfirm", { name: nickname })}
        onConfirm={() =>
          run(
            () =>
              api.DELETE(
                "/tenants/{slug}/tournaments/{tournament}/participants/{user_id}",
                {
                  params: {
                    path: { slug: tenant, tournament, user_id: userId },
                  },
                },
              ),
            () => router.refresh(),
          )
        }
        className="text-red-600 dark:text-red-400"
      >
        {t("remove")}
      </ConfirmButton>
      {error && <ErrorText size="xs">{error}</ErrorText>}
    </div>
  );
}
