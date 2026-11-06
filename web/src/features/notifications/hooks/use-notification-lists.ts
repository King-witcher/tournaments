import {
  type RefObject,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";

import { useRealtimeHub } from "@/features/realtime/hooks/use-realtime-hub";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";

export type NotificationEntry = components["schemas"]["Notification"];
type NotificationPage = components["schemas"]["NotificationPage"];
export type NotificationView = "unread" | "read";
/** O que uma vista já trouxe e o `before` da página seguinte (`null`: acabou). */
export type NotificationList = {
  items: NotificationEntry[];
  next: number | null;
};
export type NotificationLists = Partial<
  Record<NotificationView, NotificationList>
>;

/** Quantas cada busca traz: a primeira página e cada "Carregar mais". */
const PAGE_SIZE = 20;
/** O maior `limit` de `GET /me/notifications`. */
const MAX_LIMIT = 100;
/** Os avisos de um lote do despachante chegam juntos: uma busca só. */
const REFRESH_DELAY_MS = 300;

/** Só a primeira página do que já veio: cada vista mostrada começa nela. */
function firstPage(list?: NotificationList) {
  if (!list || list.items.length <= PAGE_SIZE) return list;
  const items = list.items.slice(0, PAGE_SIZE);
  return { items, next: items[items.length - 1].id };
}

function fromPage(page: NotificationPage): NotificationList {
  return { items: page.notifications, next: page.next ?? null };
}

/**
 * As listas do sininho, enquanto ele está aberto, em páginas de 20: as não
 * lidas, à mostra ao abrir, e as lidas. Busca a primeira página ao abrir e ao
 * trocar de vista, e "Carregar mais" (`loadMore`) junta a seguinte. Quando
 * chega aviso ou a contagem muda noutra aba, busca de novo de uma vez tudo o
 * que a vista mostra. Deixa em `saved` o que chegou, para a abertura seguinte
 * não começar vazia. As não lidas guardadas só valem inteiras e se batem com
 * a contagem: senão, mostrariam por um instante o que já foi lido. A lista
 * inteira das não lidas corrige a contagem (o socket pode contar um aviso
 * duas vezes ao conectar). Enquanto existe, o sininho conta como aberto, e
 * nenhum toast aparece.
 */
export function useNotificationLists(saved: RefObject<NotificationLists>) {
  const hub = useRealtimeHub();
  const [view, setView] = useState<NotificationView>("unread");
  const [lists, setLists] = useState<NotificationLists>(() => {
    const { unread, read } = saved.current;
    const whole =
      unread?.next === null && unread.items.length === hub?.getUnread();
    return {
      read: firstPage(read),
      unread: whole ? firstPage(unread) : undefined,
    };
  });
  const [failed, setFailed] = useState(false);
  const more = useApiAction();
  // A última busca do começo da lista que vale: a resposta de uma anterior é
  // descartada.
  const latest = useRef(0);
  // A lista recomeçou (outra vista, "Marcar todas"): a página seguinte que
  // estava a caminho não entra.
  const restarted = useRef(0);
  // Quantas a vista à mostra tem: o que a busca de novo traz.
  const depth = useRef(PAGE_SIZE);

  useEffect(() => hub?.openBell(), [hub]);

  useEffect(() => {
    saved.current = lists;
  }, [saved, lists]);

  useEffect(() => {
    let timer: ReturnType<typeof setTimeout> | undefined;
    const load = () => {
      const current = ++latest.current;
      api
        .GET("/me/notifications", {
          params: {
            query: { read: view === "read", limit: depth.current },
          },
        })
        .then(({ data }) => {
          if (current !== latest.current) return;
          setFailed(!data);
          if (!data) return;
          const list = fromPage(data);
          setLists((all) => ({ ...all, [view]: list }));
          if (view === "unread" && list.next === null) {
            hub?.setUnread(list.items.length);
          }
        })
        .catch(() => {
          if (current === latest.current) setFailed(true);
        });
    };
    load();
    const off = hub?.client.onMessage((message) => {
      if (message.type !== "unread" && message.type !== "notify") return;
      clearTimeout(timer);
      timer = setTimeout(load, REFRESH_DELAY_MS);
    });
    return () => {
      latest.current++;
      clearTimeout(timer);
      off?.();
    };
  }, [view, hub]);

  const show = useCallback((next: NotificationView) => {
    restarted.current++;
    depth.current = PAGE_SIZE;
    setLists((all) => ({ ...all, [next]: firstPage(all[next]) }));
    setView(next);
  }, []);

  /**
   * A página seguinte da vista à mostra, no fim da lista; `onLoaded` recebe a
   * primeira que chegou, para o foco ir até ela.
   */
  const loadMore = (onLoaded: (first: NotificationEntry) => void) => {
    const shown = view;
    const before = lists[shown]?.next;
    if (before == null) return;
    const at = restarted.current;
    more.run(
      () =>
        api.GET("/me/notifications", {
          params: {
            query: { read: shown === "read", before, limit: PAGE_SIZE },
          },
        }),
      (page) => {
        if (at !== restarted.current) return;
        depth.current = Math.min(depth.current + PAGE_SIZE, MAX_LIMIT);
        // Uma busca de novo pode ter chegado antes: só as que vêm depois da
        // última à mostra.
        const last = lists[shown]?.items.at(-1)?.id ?? before;
        const fresh = page.notifications.filter((n) => n.id < last);
        setLists((all) => ({
          ...all,
          [shown]: {
            items: [...(all[shown]?.items ?? []), ...fresh],
            next: page.next ?? null,
          },
        }));
        if (fresh.length > 0) onLoaded(fresh[0]);
      },
    );
  };

  // As lidas guardadas ficam para trás; a troca de vista busca de novo. Uma
  // busca que saiu antes de marcar traria as não lidas de volta: não vale mais.
  const clearUnread = useCallback(() => {
    latest.current++;
    restarted.current++;
    depth.current = PAGE_SIZE;
    setLists({ unread: { items: [], next: null } });
  }, []);

  return {
    view,
    show,
    lists,
    failed,
    clearUnread,
    loadMore,
    loadingMore: more.pending,
    moreError: more.error,
  };
}
