"use client";

import { useEffect, useEffectEvent } from "react";

import { useRouter } from "@/i18n/navigation";
import { useRealtimeHub } from "../hooks/use-realtime-hub";
import { createRefreshScheduler } from "../lib/refresh-scheduler";

/**
 * Abre a conexão da aba enquanto está montado (o layout o monta só para quem
 * está logado) e recarrega a página (`router.refresh()`, que mantém o estado
 * dos componentes de cliente, o foco e o que está sendo digitado) quando muda
 * algo que ela mostra, ao perder o acesso à comunidade dela e ao reconectar
 * depois de uma queda (o que mudou no meio se perdeu). Os avisos de ~1,5 s
 * viram uma recarga; com a aba escondida, recarrega ao voltar.
 */
export function LiveConnection() {
  const hub = useRealtimeHub();
  const router = useRouter();
  const refresh = useEffectEvent(() => router.refresh());

  useEffect(() => {
    if (!hub) return;
    const scheduler = createRefreshScheduler(refresh);
    const offMessage = hub.client.onMessage((message) => {
      if (hub.affectsPage(message)) scheduler.request();
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

  return null;
}
