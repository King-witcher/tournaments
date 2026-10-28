"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ErrorText } from "@/shared/ui/error-text";
import { OverflowActions } from "@/shared/ui/overflow-actions";
import styles from "./invite-actions.module.scss";

type Props = {
  slug: string;
  inviteId: number;
  /** Quem criou, para o leitor de tela saber de qual linha é o ⋯. */
  createdBy: string;
  /** Para onde vai o foco quando a linha some (o link foi revogado). */
  focusId: string;
};

/**
 * O ⋯ da linha de um convite: revogar, com a confirmação. Copiar não dá: o
 * link só existe na hora em que é gerado.
 */
export function InviteActions({ slug, inviteId, createdBy, focusId }: Props) {
  const t = useTranslations("Tenant.invites");
  const router = useRouter();
  const { run, pending, error } = useApiAction();

  return (
    <div className={styles.actions}>
      <OverflowActions
        pending={pending}
        size="sm"
        label={t("actions", { name: createdBy })}
        items={[
          {
            label: t("revoke"),
            destructive: true,
            confirm: { question: t("revokeConfirm") },
            onSelect: () =>
              run(
                () =>
                  api.DELETE("/tenants/{slug}/invites/{invite_id}", {
                    params: { path: { slug, invite_id: inviteId } },
                  }),
                () => {
                  router.refresh();
                  focusIfLost(focusId);
                },
              ),
          },
        ]}
      />
      {error && (
        <div className={styles.error}>
          <ErrorText>{error}</ErrorText>
        </div>
      )}
    </div>
  );
}
