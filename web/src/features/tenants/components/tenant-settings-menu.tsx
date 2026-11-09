"use client";

import { useTranslations } from "next-intl";

import { usePathname } from "@/i18n/navigation";
import { AppSideNavLink } from "@/shared/ui/app-side-nav-link";
import { SideNavMenu } from "@/shared/ui/carbon/ui-shell";

/**
 * Configurações da comunidade no menu lateral, para o owner: desdobra em
 * Geral, Rating e Discord (o segundo nível, como no torneio), aberta quando o
 * endereço é de uma delas. Para quem não é owner só existe Geral, e o menu
 * mostra o link simples (`TenantSideNav`).
 */
export function TenantSettingsMenu({ slug }: { slug: string }) {
  const t = useTranslations("Tenant");
  const pathname = usePathname();
  const base = `/${slug}/settings`;
  const open = pathname === base || pathname.startsWith(`${base}/`);

  return (
    // Chegar às Configurações por um link de fora do menu também o abre.
    <SideNavMenu
      key={String(open)}
      title={t("tabs.settings")}
      defaultExpanded={open}
    >
      {/* Geral é o endereço das Configurações, prefixo das outras. */}
      <AppSideNavLink href={base} match="exact" inMenu>
        {t("settings.sections.general")}
      </AppSideNavLink>
      <AppSideNavLink href={`${base}/rating`} inMenu>
        {t("settings.sections.rating")}
      </AppSideNavLink>
      <AppSideNavLink href={`${base}/discord`} inMenu>
        {t("settings.sections.discord")}
      </AppSideNavLink>
    </SideNavMenu>
  );
}
