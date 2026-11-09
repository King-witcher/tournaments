import { notFound } from "next/navigation";
import { getTranslations } from "next-intl/server";
import { getTenant } from "@/features/tenants/server";
import { apiErrorMessage } from "@/shared/api/errors";
import { FormRow } from "@/shared/ui/form-row";
import { Link } from "@/shared/ui/link";
import { Notice } from "@/shared/ui/notice";
import { Panel } from "@/shared/ui/panel";
import { Section } from "@/shared/ui/section";
import { RATING_GUIDE_HREF } from "../lib/guide-href";
import { getRatingSettings } from "../server";
import { RatingSettingsForm } from "./rating-settings-form";

/**
 * A página Rating das Configurações, só do owner, os parâmetros do Elo na
 * largura de leitura; para os outros papéis ela não existe. Sem acesso, o
 * cabeçalho da seção já mostra o erro. Fica em `<Suspense>`.
 */
export async function RatingSettingsSection({ slug }: { slug: string }) {
  const loaded = await getTenant(slug);
  if (loaded.kind !== "ok") return null;
  if (loaded.tenant.role !== "owner") notFound();
  const [settings, t] = await Promise.all([
    getRatingSettings(slug),
    getTranslations("Ratings.settings"),
  ]);
  return (
    <Panel>
      <Section
        title={t("title")}
        description={
          <Link inline href={RATING_GUIDE_HREF}>
            {t("guide")}
          </Link>
        }
      >
        {settings ? (
          <FormRow columns={2}>
            <RatingSettingsForm slug={slug} initial={settings} />
          </FormRow>
        ) : (
          <Notice kind="error">{await apiErrorMessage("internal")}</Notice>
        )}
      </Section>
    </Panel>
  );
}
