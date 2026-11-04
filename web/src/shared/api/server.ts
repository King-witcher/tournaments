import { cookies } from "next/headers";
import createClient from "openapi-fetch";

import type { paths } from "./schema";
import { API_URL } from "./url";

/**
 * Quanto uma busca do servidor espera a API. A rota mais lenta responde em
 * poucos segundos, e o cold start do Render passa de 30 s: esperar por ele
 * deixaria a página pendurada no esqueleto. Dez segundos é o que uma pessoa
 * aguenta olhando uma tela sem resposta; depois disso ela vê a falha e tenta
 * de novo, e o pedido que desistiu já acordou o servidor.
 */
const TIMEOUT_MS = 10_000;

/**
 * Client para Server Components: repassa os cookies do navegador (sessão)
 * para a API. Lê `cookies()`, então quem usa precisa estar em `<Suspense>`.
 * Passado o tempo, a busca lança o erro, como com a API fora do ar.
 */
export async function serverApi() {
  const cookieStore = await cookies();
  return createClient<paths>({
    baseUrl: API_URL,
    headers: { cookie: cookieStore.toString() },
    fetch: (request) =>
      fetch(request, { signal: AbortSignal.timeout(TIMEOUT_MS) }),
  });
}
