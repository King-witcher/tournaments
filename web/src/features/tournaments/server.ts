import { cache } from "react";

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
 * A entrada de quem vê em "Seus torneios" para este torneio: a situação (na
 * chave, eliminado, campeão…), a colocação e a próxima partida. `undefined`
 * se ele não está inscrito, se o torneio não aparece ali (rascunho) ou se a
 * API falhar. Lê a sessão: quem usa fica em `<Suspense>`.
 */
export const getMyTournamentEntry = cache(
  async (
    tenant: string,
    slug: string,
  ): Promise<Schemas["MyTournament"] | undefined> => {
    const api = await serverApi();
    const entries = await api
      .GET("/me/tournaments")
      .then(({ data }) => data ?? [])
      .catch(() => []);
    return entries.find(
      (e) => e.tenant_slug === tenant && e.tournament.slug === slug,
    );
  },
);
