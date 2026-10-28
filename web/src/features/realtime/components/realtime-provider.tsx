"use client";

import { type ReactNode, useState } from "react";

import { API_URL } from "@/shared/api/url";
import { RealtimeContext } from "../hooks/use-realtime-hub";
import { RealtimeHub } from "../lib/realtime-hub";

/** `GET /ws` na API: o endereço dela com `ws` no lugar de `http`. */
const WS_URL = `${API_URL.replace(/^http/, "ws")}/ws`;

/**
 * Guarda a conexão da aba (uma só) para as páginas declararem o que mostram
 * (`LiveUpdates`). Fica em volta do site inteiro no layout; a conexão só abre
 * com o `LiveConnection`, que o layout monta para quem está logado.
 */
export function RealtimeProvider({ children }: { children: ReactNode }) {
  const [hub] = useState(() => new RealtimeHub(WS_URL));
  return <RealtimeContext value={hub}>{children}</RealtimeContext>;
}
