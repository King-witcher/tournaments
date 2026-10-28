"use client";

import { useTranslations } from "next-intl";

import { usePathname } from "@/i18n/navigation";
import { ButtonLink } from "@/shared/ui/button-link";

/**
 * Leva ao cadastro, que depois volta para a página atual. Ao lado do "Entrar"
 * quando a página pede login: quem chega por um link compartilhado
 * geralmente ainda não tem conta.
 */
export function SignupButton() {
  const t = useTranslations("Auth");
  const pathname = usePathname();

  return (
    <ButtonLink
      href={{ pathname: "/signup", query: { return_to: pathname } }}
      kind="secondary"
    >
      {t("signup")}
    </ButtonLink>
  );
}
