"use client";

import { useTranslations } from "next-intl";
import { useCallback } from "react";

import {
  BACKUP_LOGIN_TOAST,
  useBackupLoginToast,
} from "@/features/auth/hooks/use-backup-login-toast";
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
 * Embaixo deles, na mesma pilha, o de login reserva de quem acabou de criar a
 * conta por um provedor (`useBackupLoginToast`), que fica até fechar.
 */
export function NoticeToasts({ notices, onClose }: Props) {
  const t = useTranslations("Notifications");
  const text = useNoticeText();
  const pathname = usePathname();
  const backup = useBackupLoginToast();
  const closeBackup = backup.close;
  const close = useCallback(
    (id: string) => (id === BACKUP_LOGIN_TOAST ? closeBackup() : onClose(id)),
    [closeBackup, onClose],
  );
  const arrived = notices.map(({ id, notice }) => {
    const { title, body, context, open } = text(notice);
    return {
      id,
      title,
      subtitle: body,
      caption: context,
      link:
        notice.path === pathname
          ? undefined
          : { href: notice.path, label: open },
    };
  });
  return (
    <ToastStack
      label={t("region")}
      onClose={close}
      items={backup.item ? [backup.item, ...arrived] : arrived}
    />
  );
}
