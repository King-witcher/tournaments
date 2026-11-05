"use client";

import { useTranslations } from "next-intl";
import { type RefObject, useId, useState } from "react";

import { focusIfLost } from "@/shared/lib/focus";
import { InlineLoading } from "@/shared/ui/carbon/inline-loading";
import { ContentSwitcher } from "@/shared/ui/content-switcher";
import { EmptyState } from "@/shared/ui/empty-state";
import { ErrorText } from "@/shared/ui/error-text";
import {
  type NotificationLists,
  type NotificationView,
  useNotificationLists,
} from "../hooks/use-notification-lists";
import { MarkAllReadButton } from "./mark-all-read-button";
import styles from "./notification-list.module.scss";
import { NotificationTile } from "./notification-tile";

/**
 * O conteúdo do sininho, montado a cada abertura: as não lidas por padrão e,
 * no `ContentSwitcher` (filtro da mesma lista, como o Carbon recomenda no
 * lugar de abas), as lidas. O título e o filtro não esperam a busca, então as
 * lidas estão ao alcance também sem nada não lido. O leitor de tela ouve
 * quantas há a cada troca de vista; depois de "Marcar todas", que some, o foco
 * vai para o filtro.
 */
export function NotificationList({
  saved,
}: {
  saved: RefObject<NotificationLists>;
}) {
  const t = useTranslations("Notifications.bell");
  const [view, setView] = useState<NotificationView>("unread");
  const { lists, failed, clearUnread } = useNotificationLists(view, saved);
  const id = useId();
  const titleId = `${id}-title`;
  const tabId = (name: NotificationView) => `${id}-${name}`;
  const list = lists[view];
  const empty = t(view === "read" ? "emptyRead" : "emptyUnread");

  return (
    <>
      <h2 id={titleId} className={styles.title}>
        {t("title")}
      </h2>
      <div className={styles.views}>
        <ContentSwitcher
          lowContrast
          size="sm"
          aria-labelledby={titleId}
          options={[
            { value: "unread", text: t("unread"), id: tabId("unread") },
            { value: "read", text: t("read"), id: tabId("read") },
          ]}
          value={view}
          onChange={setView}
        />
      </div>
      <div role="tabpanel" aria-labelledby={tabId(view)}>
        <p role="status" className={styles.announce}>
          {list &&
            (list.length === 0
              ? empty
              : t(view === "read" ? "countRead" : "countUnread", {
                  count: list.length,
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
        ) : list.length === 0 ? (
          <EmptyState>{empty}</EmptyState>
        ) : (
          <>
            {view === "unread" && (
              <MarkAllReadButton
                upTo={list[0].id}
                onDone={() => {
                  clearUnread();
                  focusIfLost(tabId("unread"));
                }}
              />
            )}
            <ul className={styles.list}>
              {list.map((notification) => (
                <li key={notification.id}>
                  <NotificationTile notification={notification} />
                </li>
              ))}
            </ul>
          </>
        )}
      </div>
    </>
  );
}
