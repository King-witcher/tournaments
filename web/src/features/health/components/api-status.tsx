import { getTranslations } from "next-intl/server";

import { serverApi } from "@/shared/api/server";
import { Alert } from "@/shared/ui/alert";

type Status = "ok" | "unavailable" | "offline";

async function fetchStatus(): Promise<Status> {
  try {
    const api = await serverApi();
    // 503 também traz o corpo com o status do banco.
    const { data, error } = await api.GET("/health");
    return (data ?? error)?.database ?? "offline";
  } catch {
    return "offline";
  }
}

/**
 * Consulta a API a cada requisição e só aparece quando há problema (API fora
 * ou banco indisponível): com tudo funcionando, não é assunto de quem usa.
 * Fica dentro de <Suspense>.
 */
export async function ApiStatus() {
  const [t, status] = await Promise.all([
    getTranslations("Home.status"),
    fetchStatus(),
  ]);

  if (status === "ok") return null;
  return <Alert>{t(status)}</Alert>;
}
