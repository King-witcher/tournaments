import { getTranslations } from "next-intl/server";

import { AppSideNav } from "@/shared/ui/app-side-nav";
import { AppSideNavLink } from "@/shared/ui/app-side-nav-link";
import { AppSideNavTitle } from "@/shared/ui/app-side-nav-title";
import { Avatar } from "@/shared/ui/avatar";
import { StatusTag } from "@/shared/ui/status-tag";
import { getTenant } from "../server";

/**
 * O menu lateral da comunidade: no topo, "Comunidade", o logo, o nome e o
 * papel de quem vê; as seções (Torneios, Ranking, Membros, Configurações).
 * Dentro de um torneio, o menu é o dele (`TournamentsSideNav`). Sem sessão
 * ou sem acesso, nada (a página diz o porquê). Fica em `<Suspense>`.
 */
export async function TenantSideNav({ slug }: { slug: string }) {
  const [loaded, t, tRoles] = await Promise.all([
    getTenant(slug),
    getTranslations("Tenant"),
    getTranslations("Roles"),
  ]);
  if (loaded.kind !== "ok") return null;
  const { tenant } = loaded;

  return (
    <AppSideNav
      label={t("nav")}
      header={
        <AppSideNavTitle
          kind={t("kind")}
          name={tenant.name}
          logo={
            tenant.logo_url && (
              <Avatar shape="square" size="md" src={tenant.logo_url} />
            )
          }
          tags={
            <StatusTag status="neutral" size="sm">
              {tRoles(tenant.role)}
            </StatusTag>
          }
        />
      }
    >
      {/* O início da comunidade é prefixo de todo endereço dela. */}
      <AppSideNavLink href={`/${slug}`} match="exact">
        {t("tabs.tournaments")}
      </AppSideNavLink>
      <AppSideNavLink href={`/${slug}/ranking`}>
        {t("tabs.ranking")}
      </AppSideNavLink>
      <AppSideNavLink href={`/${slug}/members`}>
        {t("tabs.members")}
      </AppSideNavLink>
      <AppSideNavLink href={`/${slug}/settings`}>
        {t("tabs.settings")}
      </AppSideNavLink>
    </AppSideNav>
  );
}
