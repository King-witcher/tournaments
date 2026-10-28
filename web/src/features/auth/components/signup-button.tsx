"use client";

import { useTranslations } from "next-intl";

import { Link, usePathname } from "@/i18n/navigation";
import { buttonVariants } from "@/shared/ui/button";

/**
 * Leva ao cadastro, que depois volta para a página atual. Ao lado do "Entrar"
 * quando a página pede login: quem chega por um link compartilhado
 * geralmente ainda não tem conta.
 */
export function SignupButton() {
  const t = useTranslations("Auth");
  const pathname = usePathname();

  return (
    <Link
      href={{ pathname: "/signup", query: { return_to: pathname } }}
      className={buttonVariants({ variant: "outline", size: "sm" })}
    >
      {t("signup")}
    </Link>
  );
}
