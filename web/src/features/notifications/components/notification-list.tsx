"use client";

import { useTranslations } from "next-intl";
import { type RefObject, useEffect, useId, useRef } from "react";

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
 * abaixo, as lidas, em páginas, com "Carregar mais" no fim (o "Load more" do
 * Carbon: o painel não tem largura para a paginação). O título e a troca de
 * vista não esperam a busca, então as lidas estão ao alcance também sem nada
 * não lido. O leitor de tela ouve quantas há a cada troca de vista e página;
 * depois de "Marcar todas", que some, o foco vai para o título, e depois de
 * "Carregar mais", para a primeira que chegou.
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
  const tileId = (notification: number) => `${id}-${notification}`;
  const list = lists[view];
  const empty = t(view === "read" ? "emptyRead" : "emptyUnread");

  // A primeira da página que chegou, para receber o foco quando aparecer.
  const focusOn = useRef<number>(undefined);
  useEffect(() => {
    if (focusOn.current === undefined || !list) return;
    document.getElementById(`${id}-${focusOn.current}`)?.focus();
    focusOn.current = undefined;
  }, [id, list]);

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
                <NotificationTile
                  id={tileId(notification.id)}
                  notification={notification}
                />
              </li>
            ))}
          </ul>
          {list.next !== null && (
            <ActionWithNote error={more.moreError}>
              <Button
                kind="ghost"
                size="sm"
                pending={more.loadingMore}
                onClick={() =>
                  more.loadMore((first) => {
                    focusOn.current = first.id;
                  })
                }
              >
                {t("more")}
              </Button>
            </ActionWithNote>
          )}
        </>
      )}
    </>
  );
}
