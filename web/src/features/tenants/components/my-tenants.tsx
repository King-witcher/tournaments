import { getTranslations } from "next-intl/server";

import { ButtonLink } from "@/shared/ui/button-link";
import { EmptyState } from "@/shared/ui/empty-state";
import { LinkTile } from "@/shared/ui/link-tile";
import { Section } from "@/shared/ui/section";
import { TileGrid } from "@/shared/ui/tile-grid";
import { getMyTenants } from "../server";

/**
 * Comunidades do usuário logado, em tiles com o papel dele, e o "Criar
 * comunidade"; sem login não mostra nada. Fica em `<Suspense>`.
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
            meta={tRoles(tenant.role)}
          />
        ))}
      </TileGrid>
    </Section>
  );
}
