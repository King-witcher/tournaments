import { getTranslations } from "next-intl/server";

import { JoinWithCodeButton } from "@/features/invites/components/join-with-code-button";
import { EmptyState } from "@/shared/ui/empty-state";
import { Section } from "@/shared/ui/section";
import { getMyTenants } from "../server";
import { CreateTenantButton } from "./create-tenant-button";
import styles from "./no-tenants.module.scss";

/**
 * Na home, para quem entrou e não participa de nenhuma comunidade: o estado,
 * "Entrar em comunidade" (quem acabou de criar conta costuma ter um convite) e o
 * "Criar comunidade", que abre o Modal. Fica em `<Suspense>`.
 */
export async function NoTenants() {
  const tenants = await getMyTenants();
  if (!tenants || tenants.length > 0) return null;
  const t = await getTranslations("Home.tenants");

  return (
    <Section title={t("title")}>
      <EmptyState
        action={
          <div className={styles.actions}>
            <JoinWithCodeButton kind="primary" />
            <CreateTenantButton kind="tertiary">
              {t("create")}
            </CreateTenantButton>
          </div>
        }
      >
        {t("empty")}
      </EmptyState>
    </Section>
  );
}
