import { cache } from "react";

import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";

/**
 * A fase seguinte à espera (da 2 em diante): quem entra, os suplentes, os
 * check-ins e se a organização pode iniciar. `undefined` se a API falhar.
 * Lê a sessão: quem usa fica em `<Suspense>`.
 */
export const loadStage = cache(
  async (
    tenant: string,
    slug: string,
    stage: number,
  ): Promise<components["schemas"]["StageEntry"] | undefined> => {
    const api = await serverApi();
    return api
      .GET("/tenants/{slug}/tournaments/{tournament}/stages/{stage}", {
        params: { path: { slug: tenant, tournament: slug, stage } },
      })
      .then(({ data }) => data)
      .catch(() => undefined);
  },
);
