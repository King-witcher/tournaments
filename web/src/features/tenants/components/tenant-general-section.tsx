import { getTranslations } from "next-intl/server";
import { CoverSection } from "@/features/images/components/cover-section";
import { Stack } from "@/shared/ui/carbon/stack";
import { FormRow } from "@/shared/ui/form-row";
import { Panel } from "@/shared/ui/panel";
import { Section } from "@/shared/ui/section";
import { getTenant } from "../server";
import { LogoSection } from "./logo-section";
import { TenantGeneralForm } from "./tenant-general-form";

/**
 * Para o owner, em Geral: nome, endereço e descrição na metade da esquerda e
 * o logo e a capa, que se salvam sozinhos, na da direita. Quem não é owner não vê nada;
 * com a exclusão agendada, nada muda, então também some. Fica em `<Suspense>`.
 */
export async function TenantGeneralSection({ slug }: { slug: string }) {
  const [loaded, t] = await Promise.all([
    getTenant(slug),
    getTranslations("Tenant.details"),
  ]);
  if (loaded.kind !== "ok" || loaded.tenant.role !== "owner") return null;
  if (loaded.tenant.deletion_scheduled_for) return null;

  return (
    <Panel>
      <Section title={t("title")}>
        <FormRow columns={2}>
          {/* Noutro endereço, os campos voltam a mostrar o de agora. */}
          <TenantGeneralForm
            key={slug}
            slug={slug}
            name={loaded.tenant.name}
            description={loaded.tenant.description ?? ""}
          />
          <Stack gap={7}>
            <LogoSection slug={slug} url={loaded.tenant.logo_url} />
            <CoverSection tenant={slug} url={loaded.tenant.cover_url} />
          </Stack>
        </FormRow>
      </Section>
    </Panel>
  );
}
