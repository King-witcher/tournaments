"use client";

import { useRouter } from "next/navigation";
import { useTranslations } from "next-intl";
import { useTransition } from "react";

import { api } from "@/lib/api/client";

export function LogoutButton() {
  const t = useTranslations("Auth");
  const router = useRouter();
  const [pending, startTransition] = useTransition();

  return (
    <button
      type="button"
      disabled={pending}
      onClick={() =>
        startTransition(async () => {
          await api.POST("/auth/logout");
          router.refresh();
        })
      }
      className="text-sm underline-offset-4 hover:underline disabled:opacity-50"
    >
      {t("logout")}
    </button>
  );
}
