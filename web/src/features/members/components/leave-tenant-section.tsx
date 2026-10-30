import { useTranslations } from "next-intl";

import { Section } from "@/shared/ui/section";
import { LeaveTenantButton } from "./leave-tenant-button";

/** Para quem não é owner: sair da comunidade, com o que acontece. */
export function LeaveTenantSection({ slug }: { slug: string }) {
  const t = useTranslations("Tenant.leave");

  return (
    <Section level={3} title={t("title")} description={t("description")}>
      <LeaveTenantButton slug={slug} />
    </Section>
  );
}
