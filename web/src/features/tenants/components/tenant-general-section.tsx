import { getTranslations } from "next-intl/server";
import { getImageSettings } from "@/features/images/server";
import { Panel } from "@/shared/ui/panel";
import { Section } from "@/shared/ui/section";
import { getTenant } from "../server";
import { TenantGeneralForm } from "./tenant-general-form";

/**
 * Para o owner, em Geral: nome, endereço e descrição na metade da esquerda e
 * o logo e a capa, que vão junto do Salvar, na da direita. Quem não é owner não
 * vê nada; com a exclusão agendada, nada muda, então também some. Fica em
 * `<Suspense>`.
 */
export async function TenantGeneralSection({ slug }: { slug: string }) {
  const [loaded, settings, t] = await Promise.all([
    getTenant(slug),
    getImageSettings(),
    getTranslations("Tenant.details"),
  ]);
  if (loaded.kind !== "ok" || loaded.tenant.role !== "owner") return null;
  if (loaded.tenant.deletion_scheduled_for) return null;

  return (
    <Panel>
      <Section title={t("title")}>
        {/* Noutro endereço, os campos voltam a mostrar o de agora. */}
        <TenantGeneralForm
          key={slug}
          slug={slug}
          name={loaded.tenant.name}
          description={loaded.tenant.description ?? ""}
          logoUrl={loaded.tenant.logo_url}
          coverUrl={loaded.tenant.cover_url}
          maxImageBytes={settings?.enabled ? settings.max_bytes : undefined}
        />
      </Section>
    </Panel>
  );
}
