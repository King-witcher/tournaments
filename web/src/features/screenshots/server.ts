import { cache } from "react";

import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";

/**
 * Se esta API tem screenshots (`GET /screenshots/settings`, pública e sem
 * sessão: não gasta comando do Redis); `null` se a API falhou (o campo some).
 * Um pedido por página. Lê os cookies: quem usa fica em `<Suspense>`.
 */
export const getScreenshotSettings = cache(
  async (): Promise<components["schemas"]["ScreenshotSettings"] | null> => {
    const api = await serverApi();
    const res = await api.GET("/screenshots/settings").catch(() => undefined);
    return res?.data ?? null;
  },
);

/**
 * Os screenshots da partida, mapa a mapa, com a URL assinada de cada um; um
 * pedido só por página (cada mapa lê daqui). `null` se a API falhou: a página
 * segue sem screenshots. Lê a sessão: quem usa fica em `<Suspense>`.
 */
export const loadMatchScreenshots = cache(
  async (
    tenant: string,
    slug: string,
    key: string,
  ): Promise<components["schemas"]["MatchScreenshots"] | null> => {
    const api = await serverApi();
    const res = await api
      .GET(
        "/tenants/{slug}/tournaments/{tournament}/matches/{match}/screenshots",
        {
          params: { path: { slug: tenant, tournament: slug, match: key } },
        },
      )
      .catch(() => undefined);
    return res?.data ?? null;
  },
);
