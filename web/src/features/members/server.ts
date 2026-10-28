import { cache } from "react";

import { serverApi } from "@/shared/api/server";

/**
 * Membros ativos do tenant, buscados uma vez por requisição (a lista e a
 * transferência de posse dividem). Lê cookies: quem usa fica em `<Suspense>`.
 */
export const getMembers = cache(async (slug: string) => {
  const api = await serverApi();
  return api
    .GET("/tenants/{slug}/members", { params: { path: { slug } } })
    .then(({ data }) => data ?? []);
});
