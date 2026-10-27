import createClient from "openapi-fetch";

import type { paths } from "./schema";

/** Client tipado da API Rust. Os tipos vêm de `bun run gen:api`. */
export const api = createClient<paths>({
  baseUrl: process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080",
});
