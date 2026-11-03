import { useEffect, useEffectEvent } from "react";

import { useMarkPageRead } from "./use-mark-page-read";
import { useRealtimeHub } from "./use-realtime-hub";
import { useUnreadCount } from "./use-unread-count";

/**
 * Abrir uma página, pelo aviso ou por qualquer outro caminho, marca como lidos
 * os avisos dela. O pedido só sai com a contagem já chegada e maior que zero,
 * para não custar um POST por navegação; numa aba aberta em segundo plano,
 * espera ela ser vista.
 */
export function useMarkVisitedPageRead(pathname: string) {
  const hub = useRealtimeHub();
  const markPageRead = useMarkPageRead();
  const counted = useUnreadCount() !== undefined;

  const visit = useEffectEvent((path: string) => {
    if (hub?.getUnread()) markPageRead(path);
  });

  useEffect(() => {
    if (!counted) return;
    if (document.visibilityState === "visible") {
      visit(pathname);
      return;
    }
    const onVisibility = () => {
      if (document.visibilityState !== "visible") return;
      document.removeEventListener("visibilitychange", onVisibility);
      visit(pathname);
    };
    document.addEventListener("visibilitychange", onVisibility);
    return () => document.removeEventListener("visibilitychange", onVisibility);
  }, [pathname, counted]);
}
