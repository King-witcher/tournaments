import { cookies } from "next/headers";
import createClient from "openapi-fetch";

import type { paths } from "./schema";
import { API_URL } from "./url";

/**
 * Client para Server Components: repassa os cookies do navegador (sessão)
 * para a API. Lê `cookies()`, então quem usa precisa estar em `<Suspense>`.
 */
export async function serverApi() {
  const cookieStore = await cookies();
  return createClient<paths>({
    baseUrl: API_URL,
    headers: { cookie: cookieStore.toString() },
  });
}
