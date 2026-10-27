"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";

type Props = {
  slug: string;
  userId: number;
  role: components["schemas"]["AssignableRole"];
};

/** Para o owner: alterna o membro entre player e organizer. */
export function MemberRoleButton({ slug, userId, role }: Props) {
  const t = useTranslations("Tenant.members");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const next = role === "player" ? "organizer" : "player";

  return (
    <div className="flex flex-col items-end">
      <Button
        variant="link"
        disabled={pending}
        onClick={() =>
          run(
            () =>
              api.PUT("/tenants/{slug}/members/{user_id}/role", {
                params: { path: { slug, user_id: userId } },
                body: { role: next },
              }),
            () => router.refresh(),
          )
        }
      >
        {next === "organizer" ? t("makeOrganizer") : t("makePlayer")}
      </Button>
      {error && <ErrorText size="xs">{error}</ErrorText>}
    </div>
  );
}
