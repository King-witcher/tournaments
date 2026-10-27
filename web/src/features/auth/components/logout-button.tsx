"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { Button } from "@/shared/ui/button";

export function LogoutButton() {
  const t = useTranslations("Auth");
  const router = useRouter();
  const { run, pending } = useApiAction();

  return (
    <Button
      variant="link"
      disabled={pending}
      onClick={() =>
        run(
          () => api.POST("/auth/logout"),
          () => router.refresh(),
        )
      }
    >
      {t("logout")}
    </Button>
  );
}
