"use client";

import { useTranslations } from "next-intl";

import { useNoticeText } from "@/features/realtime/hooks/use-notice-text";
import { LinkTile } from "@/shared/ui/link-tile";
import { TimeAgo } from "@/shared/ui/time-ago";
import type { NotificationEntry } from "./notification-list";
import styles from "./notification-tile.module.scss";

/**
 * Uma notificação do sininho: o tile inteiro leva à partida ou ao torneio,
 * com o título, a frase do aviso e há quanto tempo chegou; "Nova" na que
 * ainda não tinha sido vista ao abrir.
 */
export function NotificationTile({
  notification,
}: {
  notification: NotificationEntry;
}) {
  const t = useTranslations("Notifications.bell");
  const text = useNoticeText();
  const { title, body } = text(notification.notice);
  return (
    <LinkTile
      href={notification.notice.path}
      title={title}
      meta={
        <>
          <span className={styles.body}>{body}</span>
          <span className={styles.when}>
            {!notification.read && (
              <strong className={styles.new}>{t("new")}</strong>
            )}
            <TimeAgo value={notification.created_at} />
          </span>
        </>
      }
    />
  );
}
