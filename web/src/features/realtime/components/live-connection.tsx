"use client";

import { useCallback, useEffect, useEffectEvent, useState } from "react";

import { usePathname, useRouter } from "@/i18n/navigation";
import { playChime, unlockAudioOnGesture } from "@/shared/lib/chime";
import { useMissedNotices } from "../hooks/use-missed-notices";
import type { Notice } from "../hooks/use-notice-text";
import { useRealtimeHub } from "../hooks/use-realtime-hub";
import { createRefreshScheduler } from "../lib/refresh-scheduler";
import { soundEnabled } from "../lib/sound-preference";
import { NoticeToasts, type ShownNotice } from "./notice-toasts";

/** Avisos na tela ao mesmo tempo; o mais antigo sai. */
const MAX_NOTICES = 3;

let lastId = 0;

/**
 * Põe na pilha os avisos que chegaram, cada um com um id (só os últimos
 * ficam). O id sai fora do updater, que o React pode chamar duas vezes.
 */
function append(arrived: Notice[]) {
  const shown = arrived.map((notice) => ({ id: String(++lastId), notice }));
  return (list: ShownNotice[]) => [...list, ...shown].slice(-MAX_NOTICES);
}

/**
 * Abre a conexão da aba enquanto está montado (o layout o monta só para quem
 * está logado) e recarrega a página (`router.refresh()`, que mantém o estado
 * dos componentes de cliente, o foco e o que está sendo digitado) quando muda
 * algo que ela mostra, ao perder o acesso à comunidade dela e ao reconectar
 * depois de uma queda (o que mudou no meio se perdeu). Os avisos de ~1,5 s
 * viram uma recarga; com a aba escondida, recarrega ao voltar.
 *
 * Aviso para a pessoa (`notify`) com a aba visível: o toast e o som (se não
 * foi desligado). Com a aba escondida, `useMissedNotices`: o título da aba,
 * o som se não houver Web Push e o toast quando ela volta. Abrir a página de
 * um aviso o tira da tela.
 */
export function LiveConnection() {
  const hub = useRealtimeHub();
  const router = useRouter();
  const pathname = usePathname();
  const [notices, setNotices] = useState<ShownNotice[]>([]);
  const [seenPath, setSeenPath] = useState(pathname);
  if (seenPath !== pathname) {
    setSeenPath(pathname);
    setNotices((list) => list.filter((n) => n.notice.path !== pathname));
  }

  const miss = useMissedNotices((missed) => setNotices(append(missed)));

  const refresh = useEffectEvent(() => router.refresh());
  const show = useEffectEvent((notice: Notice) => {
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
      else if (hub.affectsPage(message)) scheduler.request();
    });
    const offReconnect = hub.client.onReconnect(scheduler.request);
    hub.client.connect();
    return () => {
      offMessage();
      offReconnect();
      scheduler.dispose();
      hub.client.close();
    };
  }, [hub]);

  return <NoticeToasts notices={notices} onClose={close} />;
}
