"use client";

import { Stack } from "@carbon/react";
import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { ErrorText } from "@/shared/ui/error-text";

type Props = {
  slug: string;
  /** A exclusão já está agendada: o botão cancela. */
  scheduled: boolean;
  /** O bloco que recebe o foco quando o botão troca. */
  focusId: string;
};

/**
 * Para o dono: agendar a exclusão da comunidade (o Modal pede o endereço
 * digitado) ou, agendada, cancelar. O botão troca no refresh, e o foco vai
 * para o bloco da exclusão. O erro (torneio em andamento) fica embaixo.
 */
export function DeleteTenantButton({ slug, scheduled, focusId }: Props) {
  const t = useTranslations("Tenant.deletion");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const path = { params: { path: { slug } } };
  const done = () => {
    router.refresh();
    focusIfLost(focusId);
  };

  return (
    <Stack gap={3}>
      <div>
        {scheduled ? (
          <Button
            kind="tertiary"
            pending={pending}
            onClick={() =>
              run(() => api.DELETE("/tenants/{slug}/deletion", path), done)
            }
          >
            {t("cancel")}
          </Button>
        ) : (
          <ConfirmButton
            size="md"
            pending={pending}
            confirmSlug={slug}
            question={t("confirm")}
            onConfirm={() =>
              run(
                () =>
                  api.POST("/tenants/{slug}/deletion", {
                    ...path,
                    body: { confirm_slug: slug },
                  }),
                done,
              )
            }
          >
            {t("submit")}
          </ConfirmButton>
        )}
      </div>
      {error && <ErrorText>{error}</ErrorText>}
    </Stack>
  );
}
