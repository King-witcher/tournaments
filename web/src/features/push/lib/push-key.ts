import { api } from "@/shared/api/client";

/** A chave VAPID, `off` sem push no servidor (503) ou o código do erro. */
export type PushKey =
  | { kind: "key"; value: string }
  | { kind: "off" }
  | { kind: "error"; code: string };

const CACHE_KEY = "push-key";
let pending: Promise<PushKey> | undefined;

/**
 * `GET /push/key`, uma vez por aba (no `sessionStorage`): a chave quase
 * nunca muda, e cada chamada lê a sessão na API. Erro de rede não fica
 * guardado.
 */
export function pushKey(): Promise<PushKey> {
  const cached = readCache();
  if (cached) return Promise.resolve(cached);
  pending ??= fetchKey().then((result) => {
    if (result.kind === "error") pending = undefined;
    else writeCache(result);
    return result;
  });
  return pending;
}

async function fetchKey(): Promise<PushKey> {
  try {
    const { data, response } = await api.GET("/push/key");
    if (data) return { kind: "key", value: data.public_key };
    if (response.status === 503) return { kind: "off" };
    return { kind: "error", code: "internal" };
  } catch {
    return { kind: "error", code: "internal" };
  }
}

function readCache(): PushKey | undefined {
  try {
    const value = sessionStorage.getItem(CACHE_KEY);
    return value ? (JSON.parse(value) as PushKey) : undefined;
  } catch {
    return undefined;
  }
}

function writeCache(key: PushKey) {
  try {
    sessionStorage.setItem(CACHE_KEY, JSON.stringify(key));
  } catch {
    // Sem armazenamento: busca de novo na próxima carga.
  }
}
