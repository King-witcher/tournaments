import { getTranslations } from "next-intl/server";

import { ButtonLink } from "@/shared/ui/button-link";
import { EmptyState } from "@/shared/ui/empty-state";
import { Section } from "@/shared/ui/section";
import { getMyTenants } from "../server";

/**
 * Na home, para quem entrou e não participa de nenhuma comunidade: o estado e
 * o "Criar comunidade". Fica em `<Suspense>`.
 */
export async function NoTenants() {
  const tenants = await getMyTenants();
  if (!tenants || tenants.length > 0) return null;
  const t = await getTranslations("Home.tenants");

  return (
    <Section title={t("title")}>
      <EmptyState action={<ButtonLink href="/new">{t("create")}</ButtonLink>}>
        {t("empty")}
      </EmptyState>
    </Section>
  );
}
