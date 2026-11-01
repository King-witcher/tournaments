import { cache } from "react";

import type { components } from "@/shared/api/schema";
import { serverApi } from "@/shared/api/server";

/**
 * Se esta API aceita logo e capa e o tamanho máximo do arquivo
 * (`GET /images/settings`, pública); `null` se a API falhou (o envio some).
 * Um pedido por página. Lê os cookies: quem usa fica em `<Suspense>`.
 */
export const getImageSettings = cache(
  async (): Promise<components["schemas"]["ImageSettings"] | null> => {
    const api = await serverApi();
    const res = await api.GET("/images/settings").catch(() => undefined);
    return res?.data ?? null;
  },
);
