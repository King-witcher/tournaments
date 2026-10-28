import { cookies } from "next/headers";
import { cache } from "react";

import { serverApi } from "@/shared/api/server";

/** O cookie que a API grava no login (`SESSION_COOKIE` em `auth/session.rs`). */
const SESSION_COOKIE = "session";

/**
 * Se o navegador mandou o cookie de sessão. Sem ele ninguém está logado, e
 * as buscas de quem vê nem perguntam à API: quem não entrou vê a home sem
 * esperar três respostas 401. Lê cookies: quem usa fica em `<Suspense>`.
 */
export async function hasSessionCookie() {
  return (await cookies()).has(SESSION_COOKIE);
}

/**
 * Usuário logado ou `undefined`, buscado uma vez por requisição (o menu e a
 * página dividem o resultado). Lê cookies: quem usa fica em `<Suspense>`.
 */
export const getCurrentUser = cache(async () => {
  if (!(await hasSessionCookie())) return undefined;
  const api = await serverApi();
  return api
    .GET("/me")
    .then(({ data }) => data)
    .catch(() => undefined);
});
