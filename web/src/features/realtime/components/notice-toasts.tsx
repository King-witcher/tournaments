"use client";

import { useTranslations } from "next-intl";

import { usePathname } from "@/i18n/navigation";
import { ToastStack } from "@/shared/ui/toast-stack";
import { type Notice, useNoticeText } from "../hooks/use-notice-text";

/** `id`: o da notificação no sininho, em texto (a chave do toast). */
export type ShownNotice = { id: string; notice: Notice };

type Props = {
  notices: ShownNotice[];
  /** Estável (`useCallback`): o tempo de cada aviso depende dele. */
  onClose: (id: string) => void;
};

/**
 * Os avisos que chegaram (com a aba escondida, quando ela volta), traduzidos.
 * O aviso inteiro leva à partida ou ao torneio (menos quando a página já é essa).
 */
export function NoticeToasts({ notices, onClose }: Props) {
  const t = useTranslations("Notifications");
  const text = useNoticeText();
  const pathname = usePathname();
  return (
    <ToastStack
      label={t("region")}
      onClose={onClose}
      items={notices.map(({ id, notice }) => {
        const { title, body, open } = text(notice);
        return {
          id,
          title,
          subtitle: body,
          link:
            notice.path === pathname
              ? undefined
              : { href: notice.path, label: open },
        };
      })}
    />
  );
}
