import { getTranslations } from "next-intl/server";

import { ButtonLink } from "@/shared/ui/button-link";
import { EmptyState } from "@/shared/ui/empty-state";
import { UserText } from "@/shared/ui/user-text";
import { getTenant } from "../server";

/**
 * A aba Sobre: a descrição da comunidade. Sem descrição, o owner vê onde
 * escrever (com a exclusão agendada, nada muda, então não há aonde ir); para
 * os outros a aba nem aparece, e o estado vazio só atende quem chega pelo
 * endereço. Fica em `<Suspense>`.
 */
export async function TenantAbout({ slug }: { slug: string }) {
  const [loaded, t] = await Promise.all([
    getTenant(slug),
    getTranslations("Tenant.about"),
  ]);
  // Sem acesso, o cabeçalho da comunidade já mostra o erro.
  if (loaded.kind !== "ok") return null;
  const { tenant } = loaded;

  if (tenant.description) return <UserText>{tenant.description}</UserText>;
  const editable = tenant.role === "owner" && !tenant.deletion_scheduled_for;
  return (
    <EmptyState
      action={
        editable && (
          <ButtonLink href={`/${slug}/settings`}>{t("add")}</ButtonLink>
        )
      }
    >
      {t("empty")}
    </EmptyState>
  );
}
