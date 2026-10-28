import { useTranslations } from "next-intl";

import { LeaveTenantButton } from "./leave-tenant-button";

/** Para quem não é owner: sair da comunidade, com o que acontece. */
export function LeaveTenantSection({ slug }: { slug: string }) {
  const t = useTranslations("Tenant.leave");

  return (
    <section className="flex flex-col gap-2">
      <h2 className="text-xl font-semibold">{t("title")}</h2>
      <p className="text-sm opacity-70">{t("description")}</p>
      <LeaveTenantButton slug={slug} />
    </section>
  );
}
