import { getTranslations } from "next-intl/server";

import { Avatar } from "@/shared/ui/avatar";
import { LinkTile } from "@/shared/ui/link-tile";
import { Section } from "@/shared/ui/section";
import { TileGrid } from "@/shared/ui/tile-grid";
import { getMyTenants } from "../server";
import { CreateTenantButton } from "./create-tenant-button";

/**
 * Comunidades do usuário logado, em tiles com o logo e o papel dele, e, depois
 * da lista, o "Criar comunidade", que abre o Modal (poucos usam, e o painel
 * do cabeçalho também o tem). Sem login ou sem nenhuma, não mostra nada: quem
 * não tem comunidade vê o `NoTenants`. Fica em `<Suspense>`.
 */
export async function MyTenants() {
  const tenants = await getMyTenants();
  if (!tenants?.length) return null;

  const [t, tRoles] = await Promise.all([
    getTranslations("Home.tenants"),
    getTranslations("Roles"),
  ]);
  // Com algum logo, todas levam um (a inicial na que não tem), e os nomes
  // ficam alinhados; sem nenhum, nenhuma.
  const withLogos = tenants.some((tenant) => tenant.logo_url);
  return (
    <Section title={t("title")}>
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
      <CreateTenantButton kind="ghost" size="sm">
        {t("create")}
      </CreateTenantButton>
    </Section>
  );
}
