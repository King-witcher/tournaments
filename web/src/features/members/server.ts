import { cache } from "react";

import { serverApi } from "@/shared/api/server";
import type { Modality } from "./lib/opponents-href";

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

/** Um id de usuário válido no endereço, ou `undefined`. */
export function parseUserId(user: string) {
  const id = Number(user);
  return /^\d+$/.test(user) && Number.isSafeInteger(id) && id > 0
    ? id
    : undefined;
}

type Loaded<T> = { ok: true; data: T } | { ok: false; code: string };

/** O resultado de uma busca, ou o código do erro (`internal` com a API fora). */
async function load<T>(
  request: Promise<{ data?: T; error?: { error: string } }>,
): Promise<Loaded<T>> {
  const result = await request.catch(() => undefined);
  if (result?.data !== undefined) return { ok: true, data: result.data };
  return { ok: false, code: result?.error?.error ?? "internal" };
}

/**
 * O membro do endereço (o cabeçalho da página dele), uma busca por
 * requisição: o cabeçalho e as abas dividem. `user` é o segmento do endereço,
 * ainda texto; inválido é `member.not_found`. Lê a sessão: quem usa fica em
 * `<Suspense>`.
 */
export const getMember = cache(async (tenant: string, user: string) => {
  const userId = parseUserId(user);
  if (userId === undefined) {
    return { ok: false as const, code: "member.not_found" };
  }
  const api = await serverApi();
  return load(
    api.GET("/tenants/{slug}/members/{user_id}", {
      params: { path: { slug: tenant, user_id: userId } },
    }),
  );
});

/** Os torneios da comunidade em que o membro se inscreveu. */
export async function getMemberTournaments(tenant: string, userId: number) {
  const api = await serverApi();
  return load(
    api.GET("/tenants/{slug}/members/{user_id}/tournaments", {
      params: { path: { slug: tenant, user_id: userId } },
    }),
  );
}

/** A carreira do membro (séries, mapas, títulos, as últimas séries). */
export async function getMemberCareer(tenant: string, userId: number) {
  const api = await serverApi();
  return load(
    api.GET("/tenants/{slug}/members/{user_id}/career", {
      params: { path: { slug: tenant, user_id: userId } },
    }),
  );
}

export type GamesFilter = {
  page?: number;
  tournament?: string;
  map?: string;
  opponent?: number;
};

/** Uma página dos mapas do membro, com o filtro. */
export async function getMemberGames(
  tenant: string,
  userId: number,
  query: GamesFilter,
) {
  const api = await serverApi();
  return load(
    api.GET("/tenants/{slug}/members/{user_id}/games", {
      params: { path: { slug: tenant, user_id: userId }, query },
    }),
  );
}

/** O confronto direto do membro com cada adversário. */
export async function getMemberOpponents(
  tenant: string,
  userId: number,
  modality?: Modality,
) {
  const api = await serverApi();
  return load(
    api.GET("/tenants/{slug}/members/{user_id}/opponents", {
      params: {
        path: { slug: tenant, user_id: userId },
        query: { modality },
      },
    }),
  );
}
