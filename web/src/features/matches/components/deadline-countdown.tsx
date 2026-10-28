"use client";

import { useRouter } from "@/i18n/navigation";
import { Countdown } from "@/shared/ui/countdown";

/**
 * Tempo até o fim da janela de contestação; quando acaba, recarrega a página
 * (a API confirma o resultado ao ler).
 */
export function DeadlineCountdown({ until }: { until: string }) {
  const router = useRouter();
  return <Countdown until={until} onExpire={() => router.refresh()} />;
}
