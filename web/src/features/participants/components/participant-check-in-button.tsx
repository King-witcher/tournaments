"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import { ConfirmButton } from "@/shared/ui/confirm-button";

type Props = {
  tenant: string;
  tournament: string;
  userId: number;
  checkedIn: boolean;
  nickname: string;
};

/** Para organizer e owner, durante o check-in: marca ou desmarca (depois de confirmar) o de um inscrito. */
export function ParticipantCheckInButton({
  tenant,
  tournament,
  userId,
  checkedIn,
  nickname,
}: Props) {
  const t = useTranslations("Participants.checkIn");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const params = { path: { slug: tenant, tournament, user_id: userId } };
  const path =
    "/tenants/{slug}/tournaments/{tournament}/participants/{user_id}/check-in";

  return (
    <div className="flex flex-col items-end">
      {checkedIn ? (
        <ConfirmButton
          variant="link"
          pending={pending}
          question={t("unmarkConfirm", { name: nickname })}
          onConfirm={() =>
            run(
              () => api.DELETE(path, { params }),
              () => router.refresh(),
            )
          }
        >
          {t("unmark")}
        </ConfirmButton>
      ) : (
        <Button
          variant="link"
          pending={pending}
          onClick={() =>
            run(
              () => api.PUT(path, { params }),
              () => router.refresh(),
            )
          }
        >
          {t("mark")}
        </Button>
      )}
      {error && <ErrorText size="xs">{error}</ErrorText>}
    </div>
  );
}
