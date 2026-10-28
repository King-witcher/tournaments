"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";

type Props = {
  tenant: string;
  slug: string;
  /** Já fez: o botão desfaz. */
  checkedIn: boolean;
};

/** O check-in de quem vê, durante o check-in: fazer ou desfazer. */
export function CheckInButton({ tenant, slug, checkedIn }: Props) {
  const t = useTranslations("Participants.checkIn");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const params = { path: { slug: tenant, tournament: slug } };
  const path = "/tenants/{slug}/tournaments/{tournament}/check-in";

  return (
    <div className="flex flex-col items-start gap-1">
      <Button
        variant={checkedIn ? "outline" : "default"}
        size={checkedIn ? "sm" : "default"}
        pending={pending}
        onClick={() =>
          run(
            () =>
              checkedIn
                ? api.DELETE(path, { params })
                : api.POST(path, { params }),
            () => router.refresh(),
          )
        }
      >
        {checkedIn ? t("undo") : t("do")}
      </Button>
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
