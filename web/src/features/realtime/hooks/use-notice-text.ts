import { useLocale, useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { formatDeadline } from "../lib/format-deadline";

export type Notice = components["schemas"]["Notice"];

/**
 * Devolve a função que traduz um aviso da API: título, frase e o texto do
 * link (para a partida ou para o torneio). Os textos ficam em
 * `Notifications.notices.<código>`, os mesmos que o service worker usa; o
 * prazo (`confirm_by`) entra como `{deadline}`, na hora de quem vê.
 */
export function useNoticeText() {
  const t = useTranslations("Notifications");
  const locale = useLocale();
  return (notice: Notice) => {
    const values = {
      ...notice.params,
      deadline:
        "confirm_by" in notice.params
          ? formatDeadline(notice.params.confirm_by, locale)
          : "",
    };
    return {
      title: t(`notices.${notice.code}.title`, values),
      body: t(`notices.${notice.code}.body`, values),
      open: notice.code.startsWith("match.")
        ? t("open.match")
        : t("open.tournament"),
    };
  };
}
