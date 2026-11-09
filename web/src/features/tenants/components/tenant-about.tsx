import { getTranslations } from "next-intl/server";
import { Panel } from "@/shared/ui/panel";

import { Section } from "@/shared/ui/section";
import { getTenant } from "../server";
import { TenantDescription } from "./tenant-description";

/**
 * "Sobre", na coluna estreita do início da comunidade: a descrição, em duas
 * linhas com "Ver mais". Sem descrição (ou sem acesso), nada. Fica em
 * `<Suspense>`.
 */
export async function TenantAbout({ slug }: { slug: string }) {
  const [loaded, t] = await Promise.all([
    getTenant(slug),
    getTranslations("Tenant.about"),
  ]);
  if (loaded.kind !== "ok" || !loaded.tenant.description) return null;

  return (
    <Panel>
      <Section title={t("title")}>
        <TenantDescription>{loaded.tenant.description}</TenantDescription>
      </Section>
    </Panel>
  );
}
