import { cache } from "react";

import { serverApi } from "@/shared/api/server";

/**
 * O ranking da comunidade, uma busca por requisição (a aba Ranking e o
 * seeding dividem), ou `undefined` sem acesso ou com a API fora. Lê a
 * sessão: quem usa fica em `<Suspense>`.
 */
export const getRanking = cache(async (slug: string) => {
  const api = await serverApi();
  return api
    .GET("/tenants/{slug}/ratings", { params: { path: { slug } } })
    .then(({ data }) => data)
    .catch(() => undefined);
});

/** O rating de um membro, ou o código do erro. */
export async function getMemberRating(slug: string, userId: number) {
  const api = await serverApi();
  const result = await api
    .GET("/tenants/{slug}/ratings/{user_id}", {
      params: { path: { slug, user_id: userId } },
    })
    .catch(() => undefined);
  if (result?.data) return { ok: true as const, rating: result.data };
  return { ok: false as const, code: result?.error?.error ?? "internal" };
}

/** O Elo da comunidade, ou `undefined` com a API fora. */
export async function getRatingSettings(slug: string) {
  const api = await serverApi();
  return api
    .GET("/tenants/{slug}/ratings/settings", { params: { path: { slug } } })
    .then(({ data }) => data)
    .catch(() => undefined);
}
