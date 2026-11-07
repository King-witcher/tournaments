import { getTranslations } from "next-intl/server";

import { AppSideNav } from "@/shared/ui/app-side-nav";
import { AppSideNavLink } from "@/shared/ui/app-side-nav-link";
import { AppSideNavTitle } from "@/shared/ui/app-side-nav-title";
import { Avatar } from "@/shared/ui/avatar";
import { getCurrentUser } from "../server";
import { AccountSideNavSwitch } from "./account-side-nav-switch";

/**
 * O menu lateral da conta, como o das configurações de conta do IBM Cloud:
 * no topo, "Conta", o avatar e o nome de quem entrou; as seções (Perfil,
 * Logins, Sessões, Notificações), só nos endereços da conta
 * (`AccountSideNavSwitch`). Sem sessão, nada (a página pede para
 * entrar). Fica em `<Suspense>`.
 */
export async function AccountSideNav() {
  const [user, t] = await Promise.all([
    getCurrentUser(),
    getTranslations("Account"),
  ]);
  if (!user) return null;

  return (
    <AccountSideNavSwitch>
      <AppSideNav
        label={t("nav")}
        header={
          <AppSideNavTitle
            kind={t("kind")}
            name={user.display_name}
            logo={
              <Avatar
                src={user.avatar_url}
                name={user.display_name}
                size="md"
              />
            }
          />
        }
      >
        {/* O Perfil é o início da conta, prefixo de todo endereço dela. */}
        <AppSideNavLink href="/account" match="exact">
          {t("tabs.profile")}
        </AppSideNavLink>
        <AppSideNavLink href="/account/logins">
          {t("tabs.logins")}
        </AppSideNavLink>
        <AppSideNavLink href="/account/sessions">
          {t("tabs.sessions")}
        </AppSideNavLink>
        <AppSideNavLink href="/account/notifications">
          {t("tabs.notifications")}
        </AppSideNavLink>
      </AppSideNav>
    </AccountSideNavSwitch>
  );
}
