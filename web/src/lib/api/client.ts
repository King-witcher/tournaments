import createClient from "openapi-fetch";

import type { paths } from "./schema";
import { API_URL } from "./url";

/**
 * Client tipado da API Rust para o navegador (manda o cookie de sessão).
 * Os tipos vêm de `bun run gen:api`. Em Server Components, use `serverApi`.
 */
export const api = createClient<paths>({
  baseUrl: API_URL,
  credentials: "include",
});
