"use client";

import { useTranslations } from "next-intl";

import { usePathname } from "@/i18n/navigation";
import { ButtonLink } from "@/shared/ui/button-link";

/** Leva para a página de login, que depois volta para a página atual. */
export function LoginButton() {
  const t = useTranslations("Auth");
  const pathname = usePathname();
  // Na própria página de login o botão só repetiria o formulário.
  if (pathname === "/login") return null;
  const onSignup = pathname === "/signup";

  return (
    <ButtonLink
      href={
        onSignup
          ? "/login"
          : { pathname: "/login", query: { return_to: pathname } }
      }
    >
      {t("login")}
    </ButtonLink>
  );
}
