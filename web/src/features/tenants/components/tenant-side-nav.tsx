import { getTranslations } from "next-intl/server";

import { getCurrentUser } from "@/features/auth/server";
import { AppSideNav } from "@/shared/ui/app-side-nav";
import { AppSideNavLink } from "@/shared/ui/app-side-nav-link";
import { AppSideNavTitle } from "@/shared/ui/app-side-nav-title";
import { Avatar } from "@/shared/ui/avatar";
import { getTenant } from "../server";
import { TenantSettingsMenu } from "./tenant-settings-menu";

/**
 * O menu lateral da comunidade: no topo, "Comunidade", o logo, o nome e o
 * papel de quem vê; as seções (Torneios, Ranking, Membros, Seu perfil e, para
 * a organização, Configurações, com o segundo nível para o owner; o
 * organizer, com a exclusão agendada, não tem o que configurar). Dentro de um
 * torneio, o menu é o dele (`TournamentsSideNav`). Sem sessão ou sem acesso,
 * nada (a página diz o porquê). Fica em `<Suspense>`.
 */
export async function TenantSideNav({ slug }: { slug: string }) {
  const [loaded, me, t, tRoles] = await Promise.all([
    getTenant(slug),
    getCurrentUser(),
    getTranslations("Tenant"),
    getTranslations("Roles"),
  ]);
  if (loaded.kind !== "ok") return null;
  const { tenant } = loaded;
  // O perfil de quem vê (o mesmo endereço de `memberPath`, sem importar
  // `members`, que já depende de `tenants`).
  const profile = me ? `/${slug}/members/${me.id}` : undefined;

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
          detail={tRoles(tenant.role)}
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
      <AppSideNavLink href={`/${slug}/members`} except={profile}>
        {t("tabs.members")}
      </AppSideNavLink>
      {profile && (
        <AppSideNavLink href={profile}>{t("tabs.profile")}</AppSideNavLink>
      )}
      {tenant.role === "owner" && <TenantSettingsMenu slug={slug} />}
      {tenant.role === "organizer" && !tenant.deletion_scheduled_for && (
        <AppSideNavLink href={`/${slug}/settings`}>
          {t("tabs.settings")}
        </AppSideNavLink>
      )}
    </AppSideNav>
  );
}
