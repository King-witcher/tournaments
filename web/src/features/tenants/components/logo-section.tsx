import { Column, Grid } from "@carbon/react";
import { getTranslations } from "next-intl/server";
import { ImageField } from "@/features/images/components/image-field";
import { getImageSettings } from "@/features/images/server";
import { Section } from "@/shared/ui/section";
import { getTenant } from "../server";

/**
 * Para o owner, na aba Configurações: o logo da comunidade, que se salva
 * sozinho. Quem não é owner, ou a API sem o bucket público, não vê nada.
 * Fica em `<Suspense>`.
 */
export async function LogoSection({ slug }: { slug: string }) {
  const [loaded, settings, t] = await Promise.all([
    getTenant(slug),
    getImageSettings(),
    getTranslations("Images.logo"),
  ]);
  if (loaded.kind !== "ok" || loaded.tenant.role !== "owner") return null;
  if (!settings?.enabled) return null;

  return (
    <Grid>
      <Column sm={4} md={8} lg={8}>
        <Section title={t("title")} description={t("hint")}>
          <ImageField
            target={{ kind: "logo", tenant: slug }}
            url={loaded.tenant.logo_url}
            maxBytes={settings.max_bytes}
          />
        </Section>
      </Column>
    </Grid>
  );
}
