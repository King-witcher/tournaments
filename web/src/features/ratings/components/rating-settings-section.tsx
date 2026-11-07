import { getTranslations } from "next-intl/server";

import { getTenant } from "@/features/tenants/server";
import { apiErrorMessage } from "@/shared/api/errors";
import { Notice } from "@/shared/ui/notice";
import { Section } from "@/shared/ui/section";
import { getRatingSettings } from "../server";
import { RatingSettingsForm } from "./rating-settings-form";

/**
 * "Rating da comunidade" na aba Configurações, só para o owner: os
 * parâmetros do Elo. Quem não é owner não vê nada. Fica em `<Suspense>`.
 */
export async function RatingSettingsSection({ slug }: { slug: string }) {
  const loaded = await getTenant(slug);
  if (loaded.kind !== "ok" || loaded.tenant.role !== "owner") return null;
  const [settings, t] = await Promise.all([
    getRatingSettings(slug),
    getTranslations("Ratings.settings"),
  ]);
  return (
    <Section title={t("title")}>
      {settings ? (
        <RatingSettingsForm slug={slug} initial={settings} />
      ) : (
        <Notice kind="error">{await apiErrorMessage("internal")}</Notice>
      )}
    </Section>
  );
}
