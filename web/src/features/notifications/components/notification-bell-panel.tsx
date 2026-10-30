"use client";

import { Notification as BellIcon } from "@carbon/icons-react";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { HeaderPanelAction } from "@/shared/ui/header-panel-action";
import { useUnreadCount } from "../hooks/use-unread-count";
import styles from "./notification-bell-panel.module.scss";
import { type NotificationEntry, NotificationList } from "./notification-list";

/** Acima disso, o número do sininho para em "99+". */
const MAX_SHOWN = 99;

/**
 * O sininho: ação do cabeçalho com o número de notificações que faltam ver
 * (da conexão da aba, que soma cada aviso) e, ao abrir, a lista das que
 * chegaram nos últimos 30 dias, inclusive as que já saíram da tela. A última
 * lista que chegou fica aqui, para a abertura seguinte não começar vazia.
 */
export function NotificationBellPanel() {
  const t = useTranslations("Notifications.bell");
  const unread = useUnreadCount() ?? 0;
  const [notifications, setNotifications] = useState<NotificationEntry[]>();
  return (
    <HeaderPanelAction
      label={unread > 0 ? t("labelUnread", { count: unread }) : t("label")}
      icon={
        <span className={styles.icon}>
          <BellIcon size={20} />
          {unread > 0 && (
            // O número já está no nome da ação, que o leitor de tela lê.
            <span className={styles.badge} aria-hidden>
              {unread > MAX_SHOWN ? `${MAX_SHOWN}+` : unread}
            </span>
          )}
        </span>
      }
    >
      <NotificationList
        notifications={notifications}
        onLoad={setNotifications}
      />
    </HeaderPanelAction>
  );
}
