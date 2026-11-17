import { notFound } from "next/navigation";
import { getTranslations } from "next-intl/server";

import { EmptyState } from "@/shared/ui/empty-state";
import { getTenant } from "../server";
import { TenantGeneralSection } from "./tenant-general-section";
import { TenantInviteSection } from "./tenant-invite-section";

/**
 * Geral, nas Configurações da comunidade, só da organização: o que o owner
 * decide sobre ela e o link de convite (a zona de perigo, a página põe
 * depois). Para o jogador a página não existe (o apelido e o Sair ficam no
 * perfil dele). Com a exclusão agendada, o organizer não tem o que mudar (o
 * menu nem mostra o item) e vê o porquê. Sem acesso, o cabeçalho da seção já
 * mostra o erro. Fica em `<Suspense>`.
 */
export async function TenantSettingsGeneral({ slug }: { slug: string }) {
  const [loaded, t] = await Promise.all([
    getTenant(slug),
    getTranslations("Tenant.settings"),
  ]);
  if (loaded.kind !== "ok") return null;
  const { tenant } = loaded;
  if (tenant.role === "player") notFound();
  if (tenant.role === "organizer" && tenant.deletion_scheduled_for) {
    return <EmptyState>{t("duringDeletion")}</EmptyState>;
  }
  return (
    <>
      <TenantGeneralSection slug={slug} />
      <TenantInviteSection slug={slug} />
    </>
  );
}
