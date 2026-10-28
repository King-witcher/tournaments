/**
 * Números que alguns erros trazem para a mensagem (ex.: `{ required, available }`)
 * e, em poucos, um texto (o `locale` da tradução recusada na criação).
 */
export type ApiErrorParams = Record<string, number | string>;

/** O que `callApi` usa da resposta do openapi-fetch. */
export type ApiResponse<T> = {
  data?: T;
  error?: { error: string; params?: ApiErrorParams | null };
  response: Response;
};

export type ApiOutcome<T> =
  | { ok: true; data: T }
  | { ok: false; code: string; params?: ApiErrorParams };

/**
 * Espera a chamada (do `api` de client.ts) e devolve o dado ou o código de
 * erro da API, com os números do erro quando houver. Nunca lança: falha de
 * rede vira "internal".
 */
export async function callApi<T>(
  request: Promise<ApiResponse<T>>,
): Promise<ApiOutcome<T>> {
  try {
    const { data, error, response } = await request;
    if (response.ok) return { ok: true, data: data as T };
    return {
      ok: false,
      code: error?.error ?? "internal",
      params: error?.params ?? undefined,
    };
  } catch {
    return { ok: false, code: "internal" };
  }
}
