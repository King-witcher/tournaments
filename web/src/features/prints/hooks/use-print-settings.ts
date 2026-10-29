"use client";

import { useEffect, useState } from "react";

import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";

type Settings = components["schemas"]["PrintSettings"];

/** Um pedido por aba: a configuração só muda com outro deploy da API. */
let request: Promise<Settings | null> | undefined;

function load() {
  request ??= api
    .GET("/prints/settings")
    .then((res) => res.data ?? null)
    .catch(() => null)
    .then((settings) => {
      // Falhou: o próximo componente tenta de novo.
      if (!settings) request = undefined;
      return settings;
    });
  return request;
}

/**
 * Se esta API tem prints (`GET /prints/settings`, pública e sem sessão: não
 * gasta comando do Redis). `null` enquanto carrega ou se falhou.
 */
export function usePrintSettings() {
  const [settings, setSettings] = useState<Settings | null>(null);
  useEffect(() => {
    let live = true;
    load().then((loaded) => {
      if (live) setSettings(loaded);
    });
    return () => {
      live = false;
    };
  }, []);
  return settings;
}
