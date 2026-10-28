"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ErrorText } from "@/shared/ui/error-text";
import {
  type OverflowAction,
  OverflowActions,
} from "@/shared/ui/overflow-actions";
import styles from "./member-actions.module.scss";

type Props = {
  slug: string;
  member: Pick<
    components["schemas"]["Member"],
    "user_id" | "nickname" | "role"
  >;
  /** Owner: alterna o membro entre jogador e organizador. */
  canChangeRole: boolean;
  /** Owner remove qualquer um; organizador, só jogadores (a API confere de novo). */
  canRemove: boolean;
  /** Para onde vai o foco quando a linha some (o membro foi removido). */
  focusId: string;
};

/**
 * O ⋯ da linha de um membro: trocar o papel e remover (com a confirmação do
 * que acontece com ele). O erro aparece junto do ⋯.
 */
export function MemberActions({
  slug,
  member,
  canChangeRole,
  canRemove,
  focusId,
}: Props) {
  const t = useTranslations("Tenant.members");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const name = member.nickname;
  const path = { params: { path: { slug, user_id: member.user_id } } };
  const items: OverflowAction[] = [];

  if (canChangeRole && member.role !== "owner") {
    const next = member.role === "player" ? "organizer" : "player";
    items.push({
      label: next === "organizer" ? t("makeOrganizer") : t("makePlayer"),
      onSelect: () =>
        run(
          () =>
            api.PUT("/tenants/{slug}/members/{user_id}/role", {
              ...path,
              body: { role: next },
            }),
          () => router.refresh(),
        ),
    });
  }
  if (canRemove) {
    items.push({
      label: t("remove"),
      destructive: true,
      confirm: {
        title: t("removeTitle", { name }),
        question: t("removeConfirm", { name }),
      },
      onSelect: () =>
        run(
          () => api.DELETE("/tenants/{slug}/members/{user_id}", path),
          () => {
            router.refresh();
            focusIfLost(focusId);
          },
        ),
    });
  }

  return (
    <div className={styles.actions}>
      <OverflowActions
        items={items}
        pending={pending}
        size="sm"
        label={t("actions", { name })}
      />
      {error && (
        <div className={styles.error}>
          <ErrorText>{error}</ErrorText>
        </div>
      )}
    </div>
  );
}
