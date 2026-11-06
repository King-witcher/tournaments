import { useCallback, useEffect, useRef, useState } from "react";

import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";

type Schemas = components["schemas"];

/**
 * O que a criação no Modal busca ao abrir, e não antes: os torneios da
 * comunidade, para "Começar de" (`undefined` enquanto chegam; vazio se a
 * busca falha), e os mapas ativos, que o envio espera (o pool lembrado e o
 * copiado só levam os que existem). Buscar no cabeçalho custaria duas
 * leituras a cada página da comunidade, para um Modal que quase nunca abre.
 */
export function useCreationContext(tenant: string) {
  const [tournaments, setTournaments] =
    useState<Schemas["TournamentSummary"][]>();
  const maps = useRef<Promise<Schemas["Map"][]>>(undefined);

  const loadMaps = useCallback(() => {
    maps.current ??= api
      .GET("/maps")
      .then(({ data }) => data ?? [])
      .catch(() => []);
    return maps.current;
  }, []);

  useEffect(() => {
    let live = true;
    loadMaps();
    api
      .GET("/tenants/{slug}/tournaments", {
        params: { path: { slug: tenant } },
      })
      .then(({ data }) => live && setTournaments(data ?? []))
      .catch(() => live && setTournaments([]));
    return () => {
      live = false;
    };
  }, [tenant, loadMaps]);

  return { tournaments, maps: loadMaps };
}
