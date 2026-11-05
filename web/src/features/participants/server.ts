import { cache } from "react";

import { hasSessionCookie } from "@/features/auth/server";
import { serverApi } from "@/shared/api/server";

/**
 * Inscritos do torneio, uma busca por requisição (a lista de inscritos e a
 * confirmação de "Iniciar" dividem o resultado); `undefined` se a API falhar
 * ou sem sessão (a rota é só de membros: nem pergunta). Lê a sessão: quem usa
 * fica em `<Suspense>`.
 */
export const getParticipants = cache(async (tenant: string, slug: string) => {
  if (!(await hasSessionCookie())) return undefined;
  const api = await serverApi();
  return api
    .GET("/tenants/{slug}/tournaments/{tournament}/participants", {
      params: { path: { slug: tenant, tournament: slug } },
    })
    .then(({ data }) => data)
    .catch(() => undefined);
});
