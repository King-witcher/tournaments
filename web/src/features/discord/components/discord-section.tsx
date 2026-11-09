import { getTranslations } from "next-intl/server";
import { getTenant } from "@/features/tenants/server";
import { serverApi } from "@/shared/api/server";
import { Column, Grid } from "@/shared/ui/carbon/grid";
import { Panel } from "@/shared/ui/panel";
import { Section } from "@/shared/ui/section";
import { DiscordForm } from "./discord-form";

/**
 * Para o owner, na aba Configurações: os avisos da comunidade no Discord por
 * webhook (o endereço, os avisos, o idioma e as menções). Quem não é owner não
 * vê nada. Fica em `<Suspense>`.
 */
export async function DiscordSection({ slug }: { slug: string }) {
  const loaded = await getTenant(slug);
  if (loaded.kind !== "ok" || loaded.tenant.role !== "owner") return null;
  const [t, api] = await Promise.all([getTranslations("Discord"), serverApi()]);
  const { data } = await api
    .GET("/tenants/{slug}/discord", { params: { path: { slug } } })
    .catch(() => ({ data: undefined }));
  if (!data) return null;

  return (
    <Grid>
      <Column sm={4} md={8} lg={8}>
        <Panel>
          <Section title={t("title")} description={t("description")}>
            <DiscordForm slug={slug} settings={data} />
          </Section>
        </Panel>
      </Column>
    </Grid>
  );
}
