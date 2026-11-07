import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { TenantSideNav } from "@/features/tenants/components/tenant-side-nav";
import { TournamentNavProvider } from "@/features/tournaments/components/tournament-nav-context";
import { TournamentsSideNav } from "@/features/tournaments/components/tournaments-side-nav";
import { AppSideNav } from "@/shared/ui/app-side-nav";

/**
 * Tudo dentro da comunidade (as seções, os torneios, a pessoa, a criação):
 * o menu lateral, que não remonta ao navegar. É o da comunidade e, dentro de
 * um torneio, o dele. Enquanto carrega, o menu vazio já ocupa o lugar, e a
 * página não pula.
 */
export default async function TenantLayout({
  params,
  children,
}: LayoutProps<"/[locale]/[tenant]">) {
  const t = await getTranslations("Tenant");
  return (
    <TournamentNavProvider>
      <Suspense fallback={<AppSideNav label={t("nav")}>{null}</AppSideNav>}>
        {params.then(({ tenant }) => (
          <TournamentsSideNav tenant={tenant}>
            <TenantSideNav slug={tenant} />
          </TournamentsSideNav>
        ))}
      </Suspense>
      {children}
    </TournamentNavProvider>
  );
}
