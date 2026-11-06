"use client";

import { useTranslations } from "next-intl";
import { type RefObject, useId } from "react";

import { useInfiniteScroll } from "@/shared/hooks/use-infinite-scroll";
import { focusIfLost } from "@/shared/lib/focus";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button } from "@/shared/ui/button";
import { InlineLoading } from "@/shared/ui/carbon/inline-loading";
import { EmptyState } from "@/shared/ui/empty-state";
import { ErrorText } from "@/shared/ui/error-text";
import {
  type NotificationLists,
  useNotificationLists,
} from "../hooks/use-notification-lists";
import { MarkAllReadButton } from "./mark-all-read-button";
import styles from "./notification-list.module.scss";
import { NotificationListHeader } from "./notification-list-header";
import { NotificationTile } from "./notification-tile";

/**
 * O conteúdo do sininho, montado a cada abertura: as não lidas e, um nível
 * abaixo, as lidas, em páginas que chegam ao rolar perto do fim. O título e
 * a troca de vista não esperam a busca, então as lidas estão ao alcance também
 * sem nada não lido. O leitor de tela ouve quantas há a cada troca de vista e
 * página; depois de "Marcar todas", que some, o foco vai para o título.
 */
export function NotificationList({
  saved,
}: {
  saved: RefObject<NotificationLists>;
}) {
  const t = useTranslations("Notifications.bell");
  const { view, show, lists, failed, clearUnread, ...more } =
    useNotificationLists(saved);
  const id = useId();
  const titleId = `${id}-title`;
  const list = lists[view];
  const empty = t(view === "read" ? "emptyRead" : "emptyUnread");
  const marker = useInfiniteScroll(
    more.loadMore,
    list?.next != null && !more.loadingMore && !more.moreFailed,
  );

  return (
    <>
      <NotificationListHeader view={view} onShow={show} titleId={titleId} />
      <p role="status" className={styles.announce}>
        {list &&
          (list.items.length === 0
            ? empty
            : t(view === "read" ? "countRead" : "countUnread", {
                count: list.items.length,
              }))}
      </p>
      {failed && (
        <div className={styles.status}>
          <ErrorText>{t("failed")}</ErrorText>
        </div>
      )}
      {!list ? (
        !failed && (
          <div className={styles.status}>
            <InlineLoading description={t("loading")} />
          </div>
        )
      ) : list.items.length === 0 ? (
        <EmptyState>{empty}</EmptyState>
      ) : (
        <>
          {view === "unread" && (
            <MarkAllReadButton
              upTo={list.items[0].id}
              onDone={() => {
                clearUnread();
                focusIfLost(titleId);
              }}
            />
          )}
          <ul className={styles.list}>
            {list.items.map((notification) => (
              <li key={notification.id}>
                <NotificationTile notification={notification} />
              </li>
            ))}
          </ul>
          {list.next !== null && (
            <div ref={marker}>
              {more.moreFailed ? (
                // Com erro, o marcador não chama de novo: quem tenta é a pessoa.
                <ActionWithNote error={t("failedMore")}>
                  <Button
                    kind="ghost"
                    size="sm"
                    pending={more.loadingMore}
                    onClick={more.loadMore}
                  >
                    {t("retry")}
                  </Button>
                </ActionWithNote>
              ) : (
                more.loadingMore && (
                  <div className={styles.status}>
                    <InlineLoading description={t("loading")} />
                  </div>
                )
              )}
            </div>
          )}
        </>
      )}
    </>
  );
}
