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
  checkedIn: boolean;
};

/** Para organizer e owner, durante o check-in: marca ou desmarca o de um inscrito. */
export function ParticipantCheckInButton({
  tenant,
  tournament,
  userId,
  checkedIn,
}: Props) {
  const t = useTranslations("Participants.checkIn");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const params = { path: { slug: tenant, tournament, user_id: userId } };
  const path =
    "/tenants/{slug}/tournaments/{tournament}/participants/{user_id}/check-in";

  return (
    <div className="flex flex-col items-end">
      <Button
        variant="link"
        disabled={pending}
        onClick={() =>
          run(
            () =>
              checkedIn
                ? api.DELETE(path, { params })
                : api.PUT(path, { params }),
            () => router.refresh(),
          )
        }
      >
        {checkedIn ? t("unmark") : t("mark")}
      </Button>
      {error && <ErrorText size="xs">{error}</ErrorText>}
    </div>
  );
}
