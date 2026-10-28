import { getTranslations } from "next-intl/server";

import { ButtonLink } from "@/shared/ui/button-link";
import { EmptyState } from "@/shared/ui/empty-state";
import { Section } from "@/shared/ui/section";
import { getMembers } from "../server";
import { TransferOwnershipForm } from "./transfer-ownership-form";

type Props = {
  slug: string;
  /** O bloco que recebe o foco depois de transferir. */
  focusId: string;
};

/**
 * Para o owner: passar a posse a outro membro (e, depois, poder sair). Sem
 * outro membro, o caminho: convidar alguém. Fica em `<Suspense>`.
 */
export async function TransferOwnershipSection({ slug, focusId }: Props) {
  const [t, members] = await Promise.all([
    getTranslations("Tenant.transfer"),
    getMembers(slug),
  ]);
  const candidates = members
    .filter((m) => m.role !== "owner")
    .map((m) => ({ userId: m.user_id, nickname: m.nickname }));

  return (
    <Section title={t("title")} description={t("description")}>
      {candidates.length === 0 ? (
        <EmptyState
          action={
            <ButtonLink href={`/${slug}/invites`} kind="tertiary">
              {t("invite")}
            </ButtonLink>
          }
        >
          {t("empty")}
        </EmptyState>
      ) : (
        <TransferOwnershipForm
          slug={slug}
          candidates={candidates}
          focusId={focusId}
        />
      )}
    </Section>
  );
}
