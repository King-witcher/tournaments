import { cache } from "react";

import { hasSessionCookie } from "@/features/auth/server";
import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";

/**
 * Comunidades de quem está logado, ou `undefined` sem login (ou com a API
 * fora). Uma busca por requisição: a home usa na lista de comunidades e na de
 * torneios. Lê a sessão: quem usa fica em `<Suspense>`.
 */
export const getMyTenants = cache(async () => {
  if (!(await hasSessionCookie())) return undefined;
  const api = await serverApi();
  return api
    .GET("/tenants")
    .then(({ data }) => data)
    .catch(() => undefined);
});

export type LoadedTenant =
  | { kind: "login" }
  | { kind: "error"; code: string }
  | { kind: "ok"; tenant: components["schemas"]["MyTenant"] };

/**
 * A comunidade do ponto de vista de quem vê (nome, papel e apelido dele), uma
 * busca por requisição: o cabeçalho com as abas e cada aba dividem o
 * resultado. Sem sessão nem pergunta à API. Lê a sessão: quem usa fica em
 * `<Suspense>`.
 */
export const getTenant = cache(async (slug: string): Promise<LoadedTenant> => {
  if (!(await hasSessionCookie())) return { kind: "login" };
  const api = await serverApi();
  const result = await api
    .GET("/tenants/{slug}", { params: { path: { slug } } })
    .catch(() => undefined);
  if (result?.response.status === 401) return { kind: "login" };
  if (!result?.data) {
    return { kind: "error", code: result?.error?.error ?? "internal" };
  }
  return { kind: "ok", tenant: result.data };
});
