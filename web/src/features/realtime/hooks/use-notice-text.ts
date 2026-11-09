import { useLocale, useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { formatDeadline } from "../lib/format-deadline";

export type Notice = components["schemas"]["Notice"];

/**
 * Devolve a função que traduz um aviso da API: título, frase e o texto do
 * link (para a partida, o torneio ou as configurações da comunidade). Os
 * textos ficam em `Notifications.notices.<código>`, os mesmos que o service
 * worker usa; o prazo (`confirm_by` ou `deletes_at`) entra como `{deadline}`,
 * na hora de quem vê, e o nome vazio (conta excluída) como "Conta excluída".
 */
export function useNoticeText() {
  const t = useTranslations("Notifications");
  const locale = useLocale();
  return (notice: Notice) => {
    const params: Record<string, string | number> = { ...notice.params };
    for (const name of ["opponent", "player", "team", "champion"]) {
      if (params[name] === "") params[name] = t("deletedAccount");
    }
    const deadline =
      "confirm_by" in notice.params
        ? notice.params.confirm_by
        : "deletes_at" in notice.params
          ? notice.params.deletes_at
          : undefined;
    const values = {
      ...params,
      deadline: deadline ? formatDeadline(deadline, locale) : "",
    };
    return {
      title: t(`notices.${notice.code}.title`, values),
      body: t(`notices.${notice.code}.body`, values),
      open: notice.code.startsWith("match.")
        ? t("open.match")
        : notice.code.startsWith("tenant.")
          ? t("open.settings")
          : t("open.tournament"),
    };
  };
}
