import { cookies } from "next/headers";
import { cache } from "react";

import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";
import { SESSION_COOKIE } from "./lib/session-cookie";

/**
 * Se o navegador mandou o cookie de sessão. Sem ele ninguém está logado, e
 * as buscas de quem vê nem perguntam à API: quem não entrou vê a home sem
 * esperar três respostas 401. Lê cookies: quem usa fica em `<Suspense>`.
 */
export async function hasSessionCookie() {
  return (await cookies()).has(SESSION_COOKIE);
}

/**
 * Quem vê a página:
 * - `signed-in`: a sessão vale;
 * - `anonymous`: sem cookie;
 * - `expired`: a API recusou o cookie (401). A pessoa é anônima, e o cookie
 *   precisa sair (`ClearExpiredSession`);
 * - `unavailable`: a API não respondeu a tempo ou falhou. Não dá para saber
 *   quem é: o cookie fica, e a pessoa não vê "Entrar" como se tivesse saído.
 */
export type Session =
  | { kind: "signed-in"; user: components["schemas"]["CurrentUser"] }
  | { kind: "anonymous" | "expired" | "unavailable" };

/**
 * A sessão de quem vê, buscada uma vez por requisição (o menu e a página
 * dividem o resultado). Lê cookies: quem usa fica em `<Suspense>`.
 */
export const getSession = cache(async (): Promise<Session> => {
  if (!(await hasSessionCookie())) return { kind: "anonymous" };
  const api = await serverApi();
  const result = await api.GET("/me").catch(() => undefined);
  if (result?.data) return { kind: "signed-in", user: result.data };
  return { kind: result?.response.status === 401 ? "expired" : "unavailable" };
});

/**
 * Usuário logado ou `undefined`, para o que só muda com alguém logado (quem
 * precisa separar sessão vencida de API fora usa `getSession`). Lê cookies:
 * quem usa fica em `<Suspense>`.
 */
export const getCurrentUser = cache(async () => {
  const session = await getSession();
  return session.kind === "signed-in" ? session.user : undefined;
});

/**
 * As formas de login da conta logada (Discord, Google, senha e email),
 * buscadas uma vez por requisição (a aba Logins e o aviso da home usam). Sem
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
 * Se a conta só entra pelo email e se esta sessão já digitou o código que prova
 * o email de agora. A API recusando, `undefined`: quem usa trata como sem prova,
 * e a API confere de novo ao mudar. Lê cookies: quem usa fica em `<Suspense>`.
 */
export const getEmailProof = cache(async () => {
  const api = await serverApi();
  const result = await api.GET("/auth/email/proof").catch(() => undefined);
  return result?.data;
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
