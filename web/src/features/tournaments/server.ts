import { cache } from "react";

import { hasSessionCookie } from "@/features/auth/server";
import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";

type Schemas = components["schemas"];

export type LoadedTournament =
  | { kind: "login" }
  | { kind: "error"; code: string }
  | { kind: "ok"; me: Schemas["MyTenant"]; tournament: Schemas["Tournament"] };

/**
 * O torneio e o tenant (com o papel de quem vê), para a página do torneio e a
 * de edição. Lê a sessão, então quem usa fica em `<Suspense>`.
 */
export const loadTournament = cache(
  async (tenant: string, slug: string): Promise<LoadedTournament> => {
    const api = await serverApi();
    const [me, found] = await Promise.all([
      api
        .GET("/tenants/{slug}", { params: { path: { slug: tenant } } })
        .catch(() => undefined),
      api
        .GET("/tenants/{slug}/tournaments/{tournament}", {
          params: { path: { slug: tenant, tournament: slug } },
        })
        .catch(() => undefined),
    ]);
    if (me?.response.status === 401) return { kind: "login" };
    if (!me?.data) {
      return { kind: "error", code: me?.error?.error ?? "internal" };
    }
    if (!found?.data) {
      return { kind: "error", code: found?.error?.error ?? "internal" };
    }
    return { kind: "ok", me: me.data, tournament: found.data };
  },
);

/**
 * Os torneios de quem vê, em todas as comunidades: em que está inscrito e em
 * que pode se inscrever. `undefined` sem login ou com a API fora. Uma busca
 * por requisição: a home usa em três seções. Lê a sessão: quem usa fica em
 * `<Suspense>`.
 */
export const getMyTournaments = cache(async () => {
  if (!(await hasSessionCookie())) return undefined;
  const api = await serverApi();
  return api
    .GET("/me/tournaments")
    .then(({ data }) => data)
    .catch(() => undefined);
});

/**
 * A entrada de quem vê neste torneio, a mesma de "Seus torneios": a situação
 * (na chave, eliminado, campeão…), a colocação e a próxima partida. Busca só
 * este torneio (`.../mine`), não a lista de todas as comunidades. `undefined`
 * sem login, se ele não está inscrito, se não vê o torneio (rascunho) ou se a
 * API falhar. Lê a sessão: quem usa fica em `<Suspense>`.
 */
export const getMyTournamentEntry = cache(
  async (
    tenant: string,
    slug: string,
  ): Promise<Schemas["MyTournament"] | undefined> => {
    if (!(await hasSessionCookie())) return undefined;
    const api = await serverApi();
    return api
      .GET("/tenants/{slug}/tournaments/{tournament}/mine", {
        params: { path: { slug: tenant, tournament: slug } },
      })
      .then(({ data }) => data)
      .catch(() => undefined);
  },
);
