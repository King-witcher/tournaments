"use client";

import { useEffect, useRef, useState } from "react";

import { API_URL } from "@/shared/api/url";

/** Espera antes de cada tentativa de reconectar; a última se repete. */
const BACKOFF_MS = [1000, 2000, 5000, 10000, 30000];
/** Com o socket fora (queda ou teto de conexões), de quanto em quanto busca a sala. */
const POLL_MS = 15_000;
/** Fechamento da API quando a sala venceu: não reconecta. */
const ROOM_GONE = 4004;
/**
 * Até quanto o espectador espera para buscar a sala depois de um aviso: num
 * chat de transmissão, são muitos ao mesmo tempo, e a API serve todos com a
 * mesma leitura se eles não chegarem juntos.
 */
const SPECTATOR_SPREAD_MS = 500;

/** 16 bytes aleatórios, como o id de `RealtimeClient` (também fora de HTTPS). */
function tabId() {
  const bytes = crypto.getRandomValues(new Uint8Array(16));
  return Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
}

type Options = {
  token: string;
  spectator: boolean;
  /** Algo mudou: busca a sala. */
  onChange: () => void;
  /** A sala venceu. */
  onGone: () => void;
};

/**
 * O socket de uma sala (`GET /veto-rooms/{token}/ws`), sem login. Só avisa: a
 * sala vem da API. Reconecta com espera crescente e, a cada vez que abre, busca
 * o que mudou no meio; enquanto está fora, busca de tempos em tempos (o navegador não
 * distingue a queda do teto de conexões). Com a aba oculta, não busca: marca que
 * ficou para trás e busca uma vez ao voltar. `headers()`: os cabeçalhos da ação
 * cuja resposta a aba aplica sozinha, que então não volta por aqui.
 */
export function useRoomSocket({ token, spectator, onChange, onGone }: Options) {
  const [id] = useState(tabId);
  const latest = useRef({ onChange, onGone });
  useEffect(() => {
    latest.current = { onChange, onGone };
  });

  useEffect(() => {
    let closed = false;
    let socket: WebSocket | undefined;
    let attempt = 0;
    let retry: ReturnType<typeof setTimeout> | undefined;
    let poll: ReturnType<typeof setInterval> | undefined;
    let stale = false;
    const url = `${API_URL.replace(/^http/, "ws")}/veto-rooms/${token}/ws?connection=${id}`;

    const fetchRoom = () => {
      if (document.visibilityState === "hidden") {
        stale = true;
        return;
      }
      latest.current.onChange();
    };
    const notify = () => {
      const delay = spectator ? Math.random() * SPECTATOR_SPREAD_MS : 0;
      setTimeout(fetchRoom, delay);
    };
    const onVisibility = () => {
      if (document.visibilityState !== "visible" || !stale) return;
      stale = false;
      notify();
    };
    const open = () => {
      if (closed) return;
      socket = new WebSocket(url);
      socket.onopen = () => {
        clearInterval(poll);
        poll = undefined;
        attempt = 0;
        // Também na primeira vez: o que mudou entre o servidor desenhar a página
        // e o socket abrir (segundos, no celular) não chegaria por aqui.
        notify();
      };
      socket.onmessage = notify;
      socket.onclose = (event) => {
        if (closed) return;
        if (event.code === ROOM_GONE) {
          latest.current.onGone();
          return;
        }
        poll ??= setInterval(fetchRoom, POLL_MS);
        const wait = BACKOFF_MS[Math.min(attempt, BACKOFF_MS.length - 1)];
        attempt++;
        retry = setTimeout(open, wait);
      };
    };
    document.addEventListener("visibilitychange", onVisibility);
    open();
    return () => {
      closed = true;
      document.removeEventListener("visibilitychange", onVisibility);
      clearTimeout(retry);
      clearInterval(poll);
      socket?.close(1000);
    };
  }, [token, spectator, id]);

  return { headers: () => ({ "x-realtime-connection": id }) };
}
