"use client";

import { useTranslations } from "next-intl";

import { Link, usePathname } from "@/i18n/navigation";
import { buttonVariants } from "@/shared/ui/button";

/** Leva para a página de login, que depois volta para a página atual. */
export function LoginButton() {
  const t = useTranslations("Auth");
  const pathname = usePathname();
  // Na própria página de login o botão só repetiria o formulário.
  if (pathname === "/login") return null;
  const onSignup = pathname === "/signup";

  return (
    <Link
      href={
        onSignup
          ? "/login"
          : { pathname: "/login", query: { return_to: pathname } }
      }
      className={buttonVariants({ size: "sm" })}
    >
      {t("login")}
    </Link>
  );
}
