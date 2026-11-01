import { useTranslations } from "next-intl";
import { useEffect, useEffectEvent, useRef } from "react";

import { receivesPush } from "@/features/push/lib/browser-subscription";
import { playChime } from "@/shared/lib/chime";
import type { NotifyMessage } from "../lib/realtime-client";
import { soundEnabled } from "../lib/sound-preference";
import { restoreTabTitle, showTabTitle } from "../lib/tab-title";
import { useNoticeText } from "./use-notice-text";

/**
 * Os avisos que chegam com a aba escondida, para quem deixa o site numa aba
 * de lado: o título da aba conta quantos chegaram e mostra o último, e o som
 * toca se este navegador não recebe Web Push (com push, a notificação do
 * sistema já avisa, com o som dela). Ao voltar a ser vista, o título volta e
 * `onReturn` recebe os avisos guardados, para virarem toasts sem som.
 * Devolve a função que guarda um aviso.
 */
export function useMissedNotices(onReturn: (notices: NotifyMessage[]) => void) {
  const t = useTranslations("Notifications");
  const text = useNoticeText();
  const missed = useRef<NotifyMessage[]>([]);

  const giveBack = useEffectEvent(() => {
    if (document.visibilityState !== "visible") return;
    if (missed.current.length === 0) return;
    const notices = missed.current;
    missed.current = [];
    restoreTabTitle();
    onReturn(notices);
  });

  useEffect(() => {
    const onVisibility = () => giveBack();
    document.addEventListener("visibilitychange", onVisibility);
    return () => {
      document.removeEventListener("visibilitychange", onVisibility);
      restoreTabTitle();
    };
  }, []);

  return (notice: NotifyMessage) => {
    missed.current.push(notice);
    showTabTitle(
      t("tabTitle", {
        count: missed.current.length,
        title: text(notice).title,
      }),
    );
    if (!soundEnabled()) return;
    receivesPush().then((push) => {
      if (!push) playChime();
    });
  };
}
