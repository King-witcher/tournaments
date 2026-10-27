"use client";

import { usePathname } from "next/navigation";
import { useTranslations } from "next-intl";

import { API_URL } from "@/shared/api/url";

/** Navega para o login da API, que volta para a página atual. */
export function LoginButton() {
  const t = useTranslations("Auth");
  const pathname = usePathname();
  const href = `${API_URL}/auth/discord/login?return_to=${encodeURIComponent(pathname)}`;

  return (
    <a
      href={href}
      className="rounded-md bg-[#5865F2] px-3 py-1.5 text-sm font-medium text-white hover:bg-[#4752C4]"
    >
      {t("login")}
    </a>
  );
}
