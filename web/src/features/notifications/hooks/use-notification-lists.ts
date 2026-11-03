import { type RefObject, useCallback, useEffect, useState } from "react";

import { useRealtimeHub } from "@/features/realtime/hooks/use-realtime-hub";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";

export type NotificationEntry = components["schemas"]["Notification"];
export type NotificationView = "unread" | "read";
export type NotificationLists = Partial<
  Record<NotificationView, NotificationEntry[]>
>;

/** O máximo que `GET /me/notifications` devolve. */
const LIST_LIMIT = 50;
/** Os avisos de um lote do despachante chegam juntos: uma busca só. */
const REFRESH_DELAY_MS = 300;

/**
 * As listas do sininho, enquanto ele está aberto. Busca a vista à mostra ao
 * abrir, ao trocar de vista e quando chega aviso ou a contagem muda noutra
 * aba, e deixa em `saved` o que chegou, para a abertura seguinte não começar
 * vazia. As não lidas guardadas só valem se batem com a contagem: senão,
 * mostrariam por um instante o que já foi lido. A lista inteira das não lidas
 * corrige a contagem (o socket pode contar um aviso duas vezes ao conectar).
 * Enquanto existe, o sininho conta como aberto, e nenhum toast aparece.
 */
export function useNotificationLists(
  view: NotificationView,
  saved: RefObject<NotificationLists>,
) {
  const hub = useRealtimeHub();
  const [lists, setLists] = useState<NotificationLists>(() => {
    const { unread, read } = saved.current;
    return {
      read,
      unread: unread?.length === hub?.getUnread() ? unread : undefined,
    };
  });
  const [failed, setFailed] = useState(false);

  useEffect(() => hub?.openBell(), [hub]);

  useEffect(() => {
    saved.current = lists;
  }, [saved, lists]);

  useEffect(() => {
    let request = 0;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const load = () => {
      const current = ++request;
      api
        .GET("/me/notifications", {
          params: { query: { read: view === "read" } },
        })
        .then(({ data }) => {
          if (current !== request) return;
          setFailed(!data);
          if (!data) return;
          setLists((all) => ({ ...all, [view]: data }));
          if (view === "unread" && data.length < LIST_LIMIT) {
            hub?.setUnread(data.length);
          }
        })
        .catch(() => {
          if (current === request) setFailed(true);
        });
    };
    load();
    const off = hub?.client.onMessage((message) => {
      if (message.type !== "unread" && message.type !== "notify") return;
      clearTimeout(timer);
      timer = setTimeout(load, REFRESH_DELAY_MS);
    });
    return () => {
      request = -1;
      clearTimeout(timer);
      off?.();
    };
  }, [view, hub]);

  // As lidas guardadas ficam para trás; a troca de vista busca de novo.
  const clearUnread = useCallback(() => setLists({ unread: [] }), []);

  return { lists, failed, clearUnread };
}
