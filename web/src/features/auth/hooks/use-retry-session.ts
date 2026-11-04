import { useTransition } from "react";

import { useRouter } from "@/i18n/navigation";

/**
 * Busca a página de novo no servidor, com `pending` enquanto espera: "Tentar
 * de novo" de quem não conseguiu falar com a API.
 */
export function useRetrySession() {
  const router = useRouter();
  const [pending, startTransition] = useTransition();
  return {
    pending,
    retry: () => startTransition(() => router.refresh()),
  };
}
