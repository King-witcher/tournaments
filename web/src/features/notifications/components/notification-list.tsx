"use client";

import { InlineLoading } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useEffect, useState } from "react";

import { useRealtimeHub } from "@/features/realtime/hooks/use-realtime-hub";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { ErrorText } from "@/shared/ui/error-text";
import styles from "./notification-list.module.scss";
import { NotificationTile } from "./notification-tile";

export type NotificationEntry = components["schemas"]["Notification"];

type Props = {
  /** O que chegou na última abertura, se houve (já como visto). */
  notifications?: NotificationEntry[];
  /** Estável (o `setState` do painel): a busca sai uma vez por abertura. */
  onLoad: (notifications: NotificationEntry[]) => void;
};

/**
 * O conteúdo do sininho, montado a cada abertura: busca a lista de novo e,
 * enquanto ela não chega, mostra a da abertura anterior (ou o carregando, na
 * primeira). Se alguma não tinha sido vista, marca como vistas até a mais
 * recente da lista (a que chegar depois continua nova); a contagem volta na
 * resposta e, nas outras abas, pelo WebSocket. As que eram novas continuam
 * marcadas até o painel fechar.
 */
export function NotificationList({ notifications, onLoad }: Props) {
  const t = useTranslations("Notifications.bell");
  const hub = useRealtimeHub();
  const [fresh, setFresh] = useState<NotificationEntry[]>();
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    let live = true;
    api
      .GET("/me/notifications")
      .then(({ data }) => {
        if (!live) return;
        if (!data) {
          setFailed(true);
          return;
        }
        setFresh(data);
        onLoad(data.map((n) => ({ ...n, read: true })));
        if (!data.some((n) => !n.read)) return;
        api
          .POST("/me/notifications/read", { body: { up_to: data[0].id } })
          .then(({ data: result }) => {
            if (result) hub?.setUnread(result.unread);
          })
          // Sem a resposta, a contagem continua e a próxima abertura tenta.
          .catch(() => {});
      })
      .catch(() => {
        if (live) setFailed(true);
      });
    return () => {
      live = false;
    };
  }, [onLoad, hub]);

  const list = fresh ?? notifications;
  if (!list) {
    return (
      <div className={styles.status}>
        {failed ? (
          <ErrorText>{t("failed")}</ErrorText>
        ) : (
          <InlineLoading description={t("loading")} />
        )}
      </div>
    );
  }
  return (
    <>
      <h2 className={styles.title}>{t("title")}</h2>
      {failed && (
        <div className={styles.status}>
          <ErrorText>{t("failed")}</ErrorText>
        </div>
      )}
      {list.length === 0 ? (
        <p className={styles.status}>{t("empty")}</p>
      ) : (
        <>
          <ul className={styles.list}>
            {list.map((notification) => (
              <li key={notification.id}>
                <NotificationTile notification={notification} />
              </li>
            ))}
          </ul>
          <p className={styles.kept}>{t("kept")}</p>
        </>
      )}
    </>
  );
}
