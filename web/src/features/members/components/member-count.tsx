import { getTranslations } from "next-intl/server";

import { getMembers } from "../server";

/**
 * Quantos membros a comunidade tem, no metadado do cabeçalho de Membros (a
 * mesma busca da tabela). Fica em `<Suspense>`.
 */
export async function MemberCount({ slug }: { slug: string }) {
  const [members, t] = await Promise.all([
    getMembers(slug).catch(() => undefined),
    getTranslations("Tenant.members"),
  ]);
  if (!members) return null;
  return t("count", { count: members.length });
}
