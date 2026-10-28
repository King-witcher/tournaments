import { getTranslations } from "next-intl/server";

import { getMembers } from "../server";
import { TransferOwnershipForm } from "./transfer-ownership-form";

/**
 * Para o owner: passar a posse a outro membro (e, depois, poder sair). Fica
 * em `<Suspense>`.
 */
export async function TransferOwnershipSection({ slug }: { slug: string }) {
  const [t, members] = await Promise.all([
    getTranslations("Tenant.transfer"),
    getMembers(slug),
  ]);
  const candidates = members
    .filter((m) => m.role !== "owner")
    .map((m) => ({ userId: m.user_id, nickname: m.nickname }));

  return (
    <section className="flex flex-col gap-2">
      <h3 className="font-semibold">{t("title")}</h3>
      <p className="text-sm text-muted">{t("description")}</p>
      {candidates.length === 0 ? (
        <p className="text-sm text-muted">{t("empty")}</p>
      ) : (
        <TransferOwnershipForm slug={slug} candidates={candidates} />
      )}
    </section>
  );
}
