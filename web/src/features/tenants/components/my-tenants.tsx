import { getTranslations } from "next-intl/server";

import { Avatar } from "@/shared/ui/avatar";
import { ButtonLink } from "@/shared/ui/button-link";
import { EmptyState } from "@/shared/ui/empty-state";
import { LinkTile } from "@/shared/ui/link-tile";
import { Section } from "@/shared/ui/section";
import { TileGrid } from "@/shared/ui/tile-grid";
import { getMyTenants } from "../server";

/**
 * Comunidades do usuário logado, em tiles com o logo e o papel dele, e o
 * "Criar comunidade"; sem login não mostra nada. Fica em `<Suspense>`.
 */
export async function MyTenants() {
  const tenants = await getMyTenants();
  if (!tenants) return null;

  const [t, tRoles] = await Promise.all([
    getTranslations("Home.tenants"),
    getTranslations("Roles"),
  ]);

  if (tenants.length === 0) {
    return (
      <Section title={t("title")}>
        <EmptyState action={<ButtonLink href="/new">{t("create")}</ButtonLink>}>
          {t("empty")}
        </EmptyState>
      </Section>
    );
  }
  // Com algum logo, todas levam um (a inicial na que não tem), e os nomes
  // ficam alinhados; sem nenhum, nenhuma.
  const withLogos = tenants.some((tenant) => tenant.logo_url);
  return (
    <Section
      title={t("title")}
      action={
        <ButtonLink href="/new" kind="tertiary" size="sm">
          {t("create")}
        </ButtonLink>
      }
    >
      <TileGrid>
        {tenants.map((tenant) => (
          <LinkTile
            key={tenant.slug}
            href={`/${tenant.slug}`}
            title={tenant.name}
            avatar={
              withLogos && (
                <Avatar
                  shape="square"
                  size="md"
                  src={tenant.logo_url}
                  name={tenant.name}
                />
              )
            }
            meta={tRoles(tenant.role)}
          />
        ))}
      </TileGrid>
    </Section>
  );
}
