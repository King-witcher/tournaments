"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { OverflowActions } from "@/shared/ui/overflow-actions";

/**
 * O ⋯ da própria página na comunidade, para quem não é owner: "Sair da
 * comunidade", com o que acontece na confirmação; saiu, vai para o início.
 * O erro aparece embaixo do ⋯.
 */
export function LeaveTenantMenu({ slug }: { slug: string }) {
  const t = useTranslations("Tenant.leave");
  const router = useRouter();
  const { run, pending, error } = useApiAction();

  return (
    <ActionWithNote error={error} compact>
      <OverflowActions
        pending={pending}
        items={[
          {
            label: t("submit"),
            destructive: true,
            confirm: { question: t("confirm") },
            onSelect: () =>
              run(
                () =>
                  api.DELETE("/tenants/{slug}/me", {
                    params: { path: { slug } },
                  }),
                () => {
                  router.push("/");
                  router.refresh();
                },
              ),
          },
        ]}
      />
    </ActionWithNote>
  );
}
