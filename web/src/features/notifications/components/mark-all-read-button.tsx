"use client";

import { useTranslations } from "next-intl";

import { useRealtimeHub } from "@/features/realtime/hooks/use-realtime-hub";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button } from "@/shared/ui/button";

type Props = {
  /** A mais recente à vista: a que chegar depois continua não lida. */
  upTo: number;
  onDone: () => void;
};

/** "Marcar todas como lidas", na lista das não lidas. */
export function MarkAllReadButton({ upTo, onDone }: Props) {
  const t = useTranslations("Notifications.bell");
  const hub = useRealtimeHub();
  const { run, pending, error } = useApiAction();
  return (
    <ActionWithNote error={error}>
      <Button
        kind="ghost"
        size="sm"
        pending={pending}
        onClick={() =>
          run(
            () => api.POST("/me/notifications/read", { body: { up_to: upTo } }),
            ({ unread }) => {
              hub?.setUnread(unread);
              onDone();
            },
          )
        }
      >
        {t("markAll")}
      </Button>
    </ActionWithNote>
  );
}
