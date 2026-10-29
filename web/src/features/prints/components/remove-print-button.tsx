"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { type PrintTarget, printParams } from "../lib/print-target";

type Props = {
  target: PrintTarget;
  printId: number;
  index: number;
  focusId: string;
};

/** Remove um print, com confirmação: sai do mapa para todos. */
export function RemovePrintButton({ target, printId, index, focusId }: Props) {
  const t = useTranslations("Prints");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const params = { path: { ...printParams(target).path, print: printId } };

  const remove = () =>
    run(
      () =>
        api.DELETE(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/prints/{print}",
          { params },
        ),
      () => {
        router.refresh();
        // O botão some com o print: o foco vai para a linha do mapa.
        focusIfLost(focusId);
      },
      () => router.refresh(),
    );

  return (
    <ActionWithNote error={error}>
      <ConfirmButton
        title={t("removeTitle", { index, number: target.number })}
        question={t("removeQuestion")}
        pending={pending}
        kind="danger--ghost"
        onConfirm={remove}
      >
        {t("remove")}
      </ConfirmButton>
    </ActionWithNote>
  );
}
