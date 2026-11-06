"use client";

import { ArrowLeft } from "@carbon/icons-react";
import { useTranslations } from "next-intl";

import { focusIfLost } from "@/shared/lib/focus";
import { Button } from "@/shared/ui/button";
import { IconButton } from "@/shared/ui/icon-button";
import type { NotificationView } from "../hooks/use-notification-lists";
import styles from "./notification-list-header.module.scss";

type Props = {
  view: NotificationView;
  onShow: (view: NotificationView) => void;
  /** O id do título, nas duas vistas: o foco vai para ele ao trocar. */
  titleId: string;
};

/**
 * O título do sininho e a troca entre as não lidas e as lidas, discreta: nas
 * não lidas, que abrem o painel, "Ver lidas" (`ghost`, com a cor de link) à
 * direita; nas lidas, um nível abaixo, a seta de voltar antes do título, como
 * no painel lateral do Carbon com passos. O botão clicado some na troca, e o
 * foco vai para o título da outra vista, que diz onde se está (na seta, o
 * tooltip dela cobriria o cabeçalho).
 */
export function NotificationListHeader({ view, onShow, titleId }: Props) {
  const t = useTranslations("Notifications.bell");
  const show = (next: NotificationView) => {
    onShow(next);
    focusIfLost(titleId);
  };
  const title = (
    <h2 id={titleId} tabIndex={-1} className={styles.title}>
      {t(view === "read" ? "titleRead" : "title")}
    </h2>
  );
  return view === "read" ? (
    <div className={styles.header}>
      <IconButton label={t("back")} size="sm" onClick={() => show("unread")}>
        <ArrowLeft />
      </IconButton>
      {title}
    </div>
  ) : (
    <div className={styles.header}>
      {title}
      <Button kind="ghost" size="sm" onClick={() => show("read")}>
        {t("showRead")}
      </Button>
    </div>
  );
}
