import { serverApi } from "@/shared/api/server";

/**
 * A sala do link `token`; `null` se não existe ou venceu. Lê cookies (o client
 * do servidor): quem usa fica em `<Suspense>`. Qualquer outro erro da API sobe
 * para a página de erro da rota.
 */
export async function getVetoRoom(token: string) {
  const api = await serverApi();
  const { data, error, response } = await api.GET("/veto-rooms/{token}", {
    params: { path: { token } },
  });
  if (data) return data;
  if (response.status === 404) return null;
  throw new Error(`sala de veto: ${error?.error ?? response.status}`);
}
