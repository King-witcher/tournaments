"use client";

import { useMarkPageRead } from "@/features/realtime/hooks/use-mark-page-read";
import { useNoticeText } from "@/features/realtime/hooks/use-notice-text";
import { usePathname } from "@/i18n/navigation";
import { LinkTile } from "@/shared/ui/link-tile";
import { TimeAgo } from "@/shared/ui/time-ago";
import type { NotificationEntry } from "../hooks/use-notification-lists";
import styles from "./notification-tile.module.scss";

/**
 * Uma notificação do sininho: o tile inteiro leva à partida ou ao torneio,
 * com o título, a frase do aviso (quando há), o torneio (ou a comunidade) e
 * há quanto tempo chegou. Abrir outra página já marca como lida
 * (`useMarkVisitedPageRead`); a página em que a pessoa já está não abre de
 * novo, então marca aqui.
 */
export function NotificationTile({
  notification: { notice, read, created_at },
}: {
  notification: NotificationEntry;
}) {
  const text = useNoticeText();
  const pathname = usePathname();
  const markPageRead = useMarkPageRead();
  const { title, body, context } = text(notice);
  return (
    <LinkTile
      href={notice.path}
      title={title}
      onClick={() => {
        if (!read && notice.path === pathname) markPageRead(pathname);
      }}
      meta={
        <>
          {body && <span className={styles.body}>{body}</span>}
          {context && <span className={styles.context}>{context}</span>}
          <span className={styles.when}>
            <TimeAgo value={created_at} />
          </span>
        </>
      }
    />
  );
}
