import { useLocale, useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { formatDeadline } from "../lib/format-deadline";

export type Notice = components["schemas"]["Notice"];

/**
 * Devolve a função que traduz um aviso da API: o título (o fato), a frase
 * (o que fazer e até quando; nem todo aviso tem), o contexto (o nome do
 * torneio, ou da comunidade, numa linha à parte, menos quando o título já o
 * tem) e o texto do link (para a partida, o torneio ou as configurações da
 * comunidade). Os textos ficam em `Notifications.notices.<código>`, os mesmos
 * que o service worker usa, com a mesma regra do contexto; o prazo
 * (`confirm_by` ou `deletes_at`) entra como `{deadline}`, na hora de quem vê,
 * e o nome vazio (conta excluída) como "Conta excluída".
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
    // O tipo das chaves sai do `pt`, onde só alguns códigos têm `body`.
    const body = `notices.${notice.code}.body` as Parameters<typeof t>[0];
    const titleTemplate = t.raw(`notices.${notice.code}.title`) as string;
    const context = ["tournament", "tenant"].find(
      (name) => name in params && !titleTemplate.includes(`{${name}}`),
    );
    return {
      title: t(`notices.${notice.code}.title`, values),
      body: t.has(body) ? t(body, values) : undefined,
      context: context ? String(params[context]) : undefined,
      open: notice.code.startsWith("match.")
        ? t("open.match")
        : notice.code.startsWith("tenant.")
          ? t("open.settings")
          : t("open.tournament"),
    };
  };
}
