import { cache } from "react";

import { serverApi } from "@/shared/api/server";

/**
 * Inscritos do torneio, uma busca por requisição (a lista de inscritos e a
 * confirmação de "Iniciar" dividem o resultado); `undefined` se a API falhar.
 * Lê a sessão: quem usa fica em `<Suspense>`.
 */
export const getParticipants = cache(async (tenant: string, slug: string) => {
  const api = await serverApi();
  return api
    .GET("/tenants/{slug}/tournaments/{tournament}/participants", {
      params: { path: { slug: tenant, tournament: slug } },
    })
    .then(({ data }) => data)
    .catch(() => undefined);
});
