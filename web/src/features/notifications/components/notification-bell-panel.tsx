"use client";

import { Notification as BellIcon } from "@carbon/icons-react";
import { useTranslations } from "next-intl";
import { useRef } from "react";

import { useUnreadCount } from "@/features/realtime/hooks/use-unread-count";
import { HeaderPanelAction } from "@/shared/ui/header-panel-action";
import type { NotificationLists } from "../hooks/use-notification-lists";
import { NotificationList } from "./notification-list";

/**
 * O sininho: ação do cabeçalho com o número de notificações que faltam ler
 * (da conexão da aba, que soma cada aviso), no indicador do Carbon, e, ao
 * abrir, as listas, inclusive o que já saiu da tela. Abrir não marca nada. As
 * últimas listas que chegaram ficam aqui, para a abertura seguinte não começar
 * vazia.
 */
export function NotificationBellPanel() {
  const t = useTranslations("Notifications.bell");
  const unread = useUnreadCount() ?? 0;
  const saved = useRef<NotificationLists>({});
  return (
    <HeaderPanelAction
      label={t("label")}
      icon={<BellIcon size={20} />}
      badgeCount={unread}
    >
      <NotificationList saved={saved} />
    </HeaderPanelAction>
  );
}
