import { notFound } from "next/navigation";
import { getTranslations } from "next-intl/server";
import { getTenant } from "@/features/tenants/server";
import { serverApi } from "@/shared/api/server";
import { FormRow } from "@/shared/ui/form-row";
import { Panel } from "@/shared/ui/panel";
import { Section } from "@/shared/ui/section";
import { DiscordForm } from "./discord-form";

/**
 * A página Discord das Configurações, só do owner: os avisos da comunidade
 * por webhook, na largura de leitura; para os outros papéis ela não existe.
 * Sem acesso, o cabeçalho da seção já mostra o erro. Fica em `<Suspense>`.
 */
export async function DiscordSection({ slug }: { slug: string }) {
  const loaded = await getTenant(slug);
  if (loaded.kind !== "ok") return null;
  if (loaded.tenant.role !== "owner") notFound();
  const [t, api] = await Promise.all([getTranslations("Discord"), serverApi()]);
  const { data } = await api
    .GET("/tenants/{slug}/discord", { params: { path: { slug } } })
    .catch(() => ({ data: undefined }));
  if (!data) return null;

  return (
    <Panel>
      <Section title={t("title")} description={t("description")}>
        <FormRow columns={2}>
          <DiscordForm slug={slug} settings={data} />
        </FormRow>
      </Section>
    </Panel>
  );
}
