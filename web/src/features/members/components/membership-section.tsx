import { useTranslations } from "next-intl";

import { LeaveTenantSection } from "./leave-tenant-section";
import { NicknameForm } from "./nickname-form";
import { TransferOwnershipSection } from "./transfer-ownership-section";

type Props = {
  slug: string;
  nickname: string;
  /** O owner transfere a posse em vez de sair. */
  owner: boolean;
};

/**
 * O que é de quem vê nesta comunidade, no fim da página: o apelido e sair (ou,
 * para o owner, transferir a posse).
 */
export function MembershipSection({ slug, nickname, owner }: Props) {
  const t = useTranslations("Tenant.membership");

  return (
    <section className="flex flex-col gap-4">
      <h2 className="text-xl font-semibold">{t("title")}</h2>
      <NicknameForm slug={slug} nickname={nickname} />
      {owner ? (
        <TransferOwnershipSection slug={slug} />
      ) : (
        <LeaveTenantSection slug={slug} />
      )}
    </section>
  );
}
