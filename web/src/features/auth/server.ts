import { cache } from "react";

import { serverApi } from "@/shared/api/server";

/**
 * Usuário logado ou `undefined`, buscado uma vez por requisição (o menu e a
 * página dividem o resultado). Lê cookies: quem usa fica em `<Suspense>`.
 */
export const getCurrentUser = cache(async () => {
  const api = await serverApi();
  return api
    .GET("/me")
    .then(({ data }) => data)
    .catch(() => undefined);
});
