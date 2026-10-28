"use client";

import { useTranslations } from "next-intl";

import { usePathname } from "@/i18n/navigation";
import { Link } from "@/shared/ui/link";
import { ToastStack } from "@/shared/ui/toast-stack";
import { type Notice, useNoticeText } from "../hooks/use-notice-text";

export type ShownNotice = { id: string; notice: Notice };

type Props = {
  notices: ShownNotice[];
  /** Estável (`useCallback`): o tempo de cada aviso depende dele. */
  onClose: (id: string) => void;
};

/**
 * Os avisos que chegaram com a aba visível, traduzidos, com o link para a
 * partida ou o torneio (menos quando a página já é essa).
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
          action: notice.path !== pathname && (
            <Link href={notice.path}>{open}</Link>
          ),
        };
      })}
    />
  );
}
