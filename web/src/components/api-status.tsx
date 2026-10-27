import { connection } from "next/server";
import { getTranslations } from "next-intl/server";

import { api } from "@/lib/api/client";

type Status = "ok" | "unavailable" | "offline";

async function fetchStatus(): Promise<Status> {
  try {
    // 503 também traz o corpo com o status do banco.
    const { data, error } = await api.GET("/health");
    return (data ?? error)?.database ?? "offline";
  } catch {
    return "offline";
  }
}

/** Consulta a API a cada requisição; fica dentro de <Suspense>. */
export async function ApiStatus() {
  await connection();
  const [t, status] = await Promise.all([
    getTranslations("Home.status"),
    fetchStatus(),
  ]);

  return <p>{t(status)}</p>;
}
