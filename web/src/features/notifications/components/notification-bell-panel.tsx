"use client";

import { Notification as BellIcon } from "@carbon/icons-react";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { HeaderPanelAction } from "@/shared/ui/header-panel-action";
import { useUnreadCount } from "../hooks/use-unread-count";
import { type NotificationEntry, NotificationList } from "./notification-list";

/**
 * O sininho: ação do cabeçalho com o número de notificações que faltam ver
 * (da conexão da aba, que soma cada aviso), no indicador do Carbon, e, ao
 * abrir, a lista das que chegaram nos últimos 30 dias, inclusive as que já
 * saíram da tela. A última lista que chegou fica aqui, para a abertura
 * seguinte não começar vazia.
 */
export function NotificationBellPanel() {
  const t = useTranslations("Notifications.bell");
  const unread = useUnreadCount() ?? 0;
  const [notifications, setNotifications] = useState<NotificationEntry[]>();
  return (
    <HeaderPanelAction
      label={t("label")}
      icon={<BellIcon size={20} />}
      badgeCount={unread}
    >
      <NotificationList
        notifications={notifications}
        onLoad={setNotifications}
      />
    </HeaderPanelAction>
  );
}
