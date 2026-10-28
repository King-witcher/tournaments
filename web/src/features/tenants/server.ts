import { cache } from "react";

import { hasSessionCookie } from "@/features/auth/server";
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
