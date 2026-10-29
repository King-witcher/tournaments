"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import {
  type ScreenshotTarget,
  screenshotParams,
} from "../lib/screenshot-target";

type Props = {
  target: ScreenshotTarget;
  screenshotId: number;
  index: number;
  focusId: string;
};

/** Remove um screenshot, com confirmação: sai do mapa para todos. */
export function RemoveScreenshotButton({
  target,
  screenshotId,
  index,
  focusId,
}: Props) {
  const t = useTranslations("Screenshots");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const params = {
    path: { ...screenshotParams(target).path, screenshot: screenshotId },
  };

  const remove = () =>
    run(
      () =>
        api.DELETE(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/screenshots/{screenshot}",
          { params },
        ),
      () => {
        router.refresh();
        // O botão some com o screenshot: o foco vai para a linha do mapa.
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
