"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button } from "@/shared/ui/button";
import { ConfirmButton } from "@/shared/ui/confirm-button";

type Props = {
  slug: string;
  /** A exclusão já está agendada: o botão cancela. */
  scheduled: boolean;
  /** Nenhum torneio começou: confirmar apaga na hora, e o Modal diz isso. */
  now: boolean;
  /** Por que não dá para excluir agora (torneio em andamento). */
  blockedReason?: string;
  /** O bloco que recebe o foco quando o botão troca. */
  focusId: string;
};

/**
 * Para o dono: excluir a comunidade (o Modal pede o endereço digitado e diz se
 * ela sai na hora ou em 7 dias; quem decide é o servidor) ou, agendada,
 * cancelar. Agendou ou cancelou, o botão troca no refresh e o foco vai para o
 * bloco da exclusão; saiu na hora, o dono vai para o Início. O motivo de não
 * dar e o erro ficam embaixo.
 */
export function DeleteTenantButton({
  slug,
  scheduled,
  now,
  blockedReason,
  focusId,
}: Props) {
  const t = useTranslations("Tenant.deletion");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const path = { params: { path: { slug } } };
  const done = () => {
    router.refresh();
    focusIfLost(focusId);
  };

  return (
    <ActionWithNote note={blockedReason} error={error}>
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
          disabled={blockedReason !== undefined}
          confirmSlug={slug}
          question={t(now ? "confirmNow" : "confirm")}
          onConfirm={() =>
            run(
              () =>
                api.POST("/tenants/{slug}/deletion", {
                  ...path,
                  body: { confirm_slug: slug },
                }),
              (result) => {
                if (result.outcome === "deleted") {
                  router.push("/");
                  router.refresh();
                } else {
                  done();
                }
              },
            )
          }
        >
          {t("submit")}
        </ConfirmButton>
      )}
    </ActionWithNote>
  );
}
