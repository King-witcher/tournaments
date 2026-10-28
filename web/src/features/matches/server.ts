import { cache } from "react";

import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";

type Schemas = components["schemas"];

type Loaded<T> = { kind: "ok"; data: T } | { kind: "error"; code: string };

async function load<T>(
  request: Promise<{ data?: T; error?: { error: string } }>,
): Promise<Loaded<T>> {
  const res = await request.catch(() => undefined);
  if (!res?.data)
    return { kind: "error", code: res?.error?.error ?? "internal" };
  return { kind: "ok", data: res.data };
}

/**
 * A chave do torneio (com as confirmações por tempo já aplicadas), para a
 * seção da chave e para o desfazer início, que mostra quantos resultados
 * seriam apagados. Lê a sessão: quem usa fica em `<Suspense>`.
 */
export const loadBracket = cache(async (tenant: string, slug: string) => {
  const api = await serverApi();
  return load<Schemas["TournamentBracket"]>(
    api.GET("/tenants/{slug}/tournaments/{tournament}/bracket", {
      params: { path: { slug: tenant, tournament: slug } },
    }),
  );
});

/** Uma partida, com detalhe por mapa e histórico. */
export const loadMatch = cache(
  async (tenant: string, slug: string, key: string) => {
    const api = await serverApi();
    return load<Schemas["MatchDetail"]>(
      api.GET("/tenants/{slug}/tournaments/{tournament}/matches/{match}", {
        params: { path: { slug: tenant, tournament: slug, match: key } },
      }),
    );
  },
);

/** Mapas ativos, do pool de duelo primeiro. */
export const loadMaps = cache(async () => {
  const api = await serverApi();
  const res = await api.GET("/maps").catch(() => undefined);
  return res?.data ?? [];
});
