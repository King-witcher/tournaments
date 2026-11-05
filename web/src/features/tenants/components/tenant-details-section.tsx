import { getTranslations } from "next-intl/server";

import { Column, Grid } from "@/shared/ui/carbon/grid";
import { Section } from "@/shared/ui/section";
import { getTenant } from "../server";
import { TenantDetailsForm } from "./tenant-details-form";

/**
 * Para o owner, na aba Configurações: o nome e a descrição da comunidade. Quem
 * não é owner não vê nada; com a exclusão agendada, nada muda, então também
 * some (como a transferência de posse). Fica em `<Suspense>`.
 */
export async function TenantDetailsSection({ slug }: { slug: string }) {
  const [loaded, t] = await Promise.all([
    getTenant(slug),
    getTranslations("Tenant.details"),
  ]);
  if (loaded.kind !== "ok" || loaded.tenant.role !== "owner") return null;
  if (loaded.tenant.deletion_scheduled_for) return null;

  return (
    <Grid>
      <Column sm={4} md={8} lg={8}>
        <Section title={t("title")}>
          <TenantDetailsForm
            slug={slug}
            name={loaded.tenant.name}
            description={loaded.tenant.description ?? ""}
          />
        </Section>
      </Column>
    </Grid>
  );
}
