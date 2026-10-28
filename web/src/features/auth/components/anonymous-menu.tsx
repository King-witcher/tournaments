"use client";

import { useTranslations } from "next-intl";

import { usePathname } from "@/i18n/navigation";
import { UserPanel, type UserPanelItem } from "@/shared/ui/user-panel";

/**
 * O painel de quem não entrou: Entrar e Criar conta, que depois voltam para
 * a página atual. Na própria página de login (ou de cadastro) o item dela
 * sai, porque só repetiria o formulário.
 */
export function AnonymousMenu() {
  const t = useTranslations("Auth");
  const pathname = usePathname();
  const onAuthPage = pathname === "/login" || pathname === "/signup";
  const query = onAuthPage ? undefined : { return_to: pathname };

  const items: UserPanelItem[] = [];
  if (pathname !== "/login")
    items.push({ label: t("login"), href: { pathname: "/login", query } });
  if (pathname !== "/signup")
    items.push({ label: t("signup"), href: { pathname: "/signup", query } });

  return <UserPanel label={t("anonymousMenu")} items={items} />;
}
