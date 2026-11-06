import { getTranslations } from "next-intl/server";

import { EmptyState } from "@/shared/ui/empty-state";
import { Section } from "@/shared/ui/section";
import { getMyTenants } from "../server";
import { CreateTenantButton } from "./create-tenant-button";

/**
 * Na home, para quem entrou e não participa de nenhuma comunidade: o estado e
 * o "Criar comunidade", que abre o Modal. Fica em `<Suspense>`.
 */
export async function NoTenants() {
  const tenants = await getMyTenants();
  if (!tenants || tenants.length > 0) return null;
  const t = await getTranslations("Home.tenants");

  return (
    <Section title={t("title")}>
      <EmptyState
        action={<CreateTenantButton>{t("create")}</CreateTenantButton>}
      >
        {t("empty")}
      </EmptyState>
    </Section>
  );
}
