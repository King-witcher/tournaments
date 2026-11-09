import { getTranslations } from "next-intl/server";
import { getTenant } from "@/features/tenants/server";
import { Panel } from "@/shared/ui/panel";
import { Section } from "@/shared/ui/section";
import { NicknameForm } from "./nickname-form";

/**
 * "Seu apelido", nas Configurações da comunidade: o que é de quem vê, para
 * qualquer papel. Fica em `<Suspense>`.
 */
export async function NicknameSection({ slug }: { slug: string }) {
  const [loaded, t] = await Promise.all([
    getTenant(slug),
    getTranslations("Tenant.nickname"),
  ]);
  // Sem acesso, o cabeçalho da seção já mostra o erro.
  if (loaded.kind !== "ok") return null;

  return (
    <Panel>
      <Section title={t("title")}>
        <NicknameForm slug={slug} nickname={loaded.tenant.nickname} />
      </Section>
    </Panel>
  );
}
