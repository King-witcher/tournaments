"use client";

import { useCallback, useEffect, useEffectEvent, useState } from "react";

import { usePathname, useRouter } from "@/i18n/navigation";
import { playChime, unlockAudioOnGesture } from "@/shared/lib/chime";
import { useMarkSeen } from "../hooks/use-mark-seen";
import { useMissedNotices } from "../hooks/use-missed-notices";
import { useRealtimeHub } from "../hooks/use-realtime-hub";
import { movedPath } from "../lib/moved-path";
import type { NotifyMessage, ServerMessage } from "../lib/realtime-client";
import { createRefreshScheduler } from "../lib/refresh-scheduler";
import { soundEnabled } from "../lib/sound-preference";
import { NoticeToasts, type ShownNotice } from "./notice-toasts";

/** Avisos na tela ao mesmo tempo; o mais antigo sai. */
const MAX_NOTICES = 3;

/** Põe na pilha os avisos que chegaram (só os últimos ficam). */
function append(arrived: NotifyMessage[]) {
  const shown = arrived.map((notice) => ({ id: String(notice.id), notice }));
  return (list: ShownNotice[]) => [...list, ...shown].slice(-MAX_NOTICES);
}

/**
 * Abre a conexão da aba enquanto está montado (o layout o monta só para quem
 * está logado) e recarrega a página (`router.refresh()`, que mantém o estado
 * dos componentes de cliente, o foco e o que está sendo digitado) quando muda
 * algo que ela mostra, ao perder o acesso à comunidade dela e ao reconectar
 * depois de uma queda (o que mudou no meio se perdeu). Os avisos de ~1,5 s
 * (0,5 s numa página ao vivo, como o veto) viram uma recarga; com a aba
 * escondida, recarrega ao voltar. Se o endereço da página mudou (`moved`), vai
 * para o novo, na mesma aba e sem aviso.
 *
 * Aviso para a pessoa (`notify`) com a aba visível: o toast e o som (se não
 * foi desligado). Com a aba escondida, `useMissedNotices`: o título da aba,
 * o som se não houver Web Push e o toast quando ela volta. Fechar o toast no
 * X ou abrir a página do aviso (pelo link ou não) o tira da tela e marca a
 * notificação como vista no sininho; o que sai pelo tempo continua nova.
 */
export function LiveConnection() {
  const hub = useRealtimeHub();
  const router = useRouter();
  const pathname = usePathname();
  const [notices, setNotices] = useState<ShownNotice[]>([]);
  const markSeen = useMarkSeen();

  const open = useEffectEvent((path: string) => {
    const opened = notices.filter((n) => n.notice.path === path);
    if (opened.length === 0) return;
    setNotices((list) => list.filter((n) => n.notice.path !== path));
    markSeen(opened.map((n) => n.id));
  });
  useEffect(() => {
    open(pathname);
  }, [pathname]);

  const miss = useMissedNotices((missed) => setNotices(append(missed)));

  const refresh = useEffectEvent(() => router.refresh());
  const follow = useEffectEvent(
    (moved: Extract<ServerMessage, { type: "moved" }>) => {
      const target = movedPath(pathname, moved);
      if (target) router.replace(`${target}${window.location.search}`);
    },
  );
  const show = useEffectEvent((notice: NotifyMessage) => {
    if (document.visibilityState !== "visible") {
      miss(notice);
      return;
    }
    setNotices(append([notice]));
    if (soundEnabled()) playChime();
  });
  const close = useCallback(
    (id: string) => setNotices((list) => list.filter((n) => n.id !== id)),
    [],
  );

  useEffect(() => unlockAudioOnGesture(), []);

  useEffect(() => {
    if (!hub) return;
    const scheduler = createRefreshScheduler(refresh);
    const offMessage = hub.client.onMessage((message) => {
      if (message.type === "notify") show(message);
      else if (message.type === "moved") follow(message);
      else if (hub.affectsPage(message))
        scheduler.request(hub.affectsLive(message));
    });
    const offReconnect = hub.client.onReconnect(() => scheduler.request());
    hub.client.connect();
    return () => {
      offMessage();
      offReconnect();
      scheduler.dispose();
      hub.client.close();
      hub.setUnread(undefined);
    };
  }, [hub]);

  return (
    <NoticeToasts
      notices={notices}
      onClose={close}
      onDismiss={(id) => markSeen([id])}
    />
  );
}
