import { Column, Grid } from "@carbon/react";
import { getTranslations } from "next-intl/server";

import { Section } from "@/shared/ui/section";
import { getTenant } from "../server";
import { TenantAddressForm } from "./tenant-address-form";

/**
 * Para o owner, na aba Configurações: o endereço da comunidade, à parte do
 * nome porque pede confirmação. Quem não é owner não vê nada; com a exclusão
 * agendada, nada muda, então também some. Fica em `<Suspense>`.
 */
export async function TenantAddressSection({ slug }: { slug: string }) {
  const [loaded, t] = await Promise.all([
    getTenant(slug),
    getTranslations("Tenant.address"),
  ]);
  if (loaded.kind !== "ok" || loaded.tenant.role !== "owner") return null;
  if (loaded.tenant.deletion_scheduled_for) return null;

  return (
    <Grid>
      <Column sm={4} md={8} lg={8}>
        <Section title={t("title")}>
          {/* Noutro endereço, o campo volta a mostrar o de agora. */}
          <TenantAddressForm key={slug} slug={slug} />
        </Section>
      </Column>
    </Grid>
  );
}
