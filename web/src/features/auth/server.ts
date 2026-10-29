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

/**
 * As formas de login da conta logada (Discord, Google e nome de usuário),
 * buscadas uma vez por requisição (as abas Logins e Senha da conta usam). Sem
 * sessão, `undefined`; a API recusando, o código do erro. Lê cookies: quem
 * usa fica em `<Suspense>`.
 */
export const getLogins = cache(async () => {
  if (!(await hasSessionCookie())) return undefined;
  const api = await serverApi();
  const result = await api.GET("/auth/logins").catch(() => undefined);
  if (result?.data) return { ok: true as const, logins: result.data };
  return { ok: false as const, code: result?.error?.error ?? "internal" };
});

/**
 * As sessões abertas da conta logada (a atual marcada), buscadas uma vez por
 * requisição. Sem sessão, `undefined`; a API recusando, o código do erro. Lê
 * cookies: quem usa fica em `<Suspense>`.
 */
export const getSessions = cache(async () => {
  if (!(await hasSessionCookie())) return undefined;
  const api = await serverApi();
  const result = await api.GET("/auth/sessions").catch(() => undefined);
  if (result?.data) return { ok: true as const, sessions: result.data };
  return { ok: false as const, code: result?.error?.error ?? "internal" };
});
