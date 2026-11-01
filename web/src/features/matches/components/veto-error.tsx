"use client";

import { ErrorText } from "@/shared/ui/error-text";
import { useLiveMatch } from "../hooks/use-live-match";

/**
 * O erro da última ação do veto, uma vez por passo à vista (no fim do
 * passo; nas Partidas, no cartão da partida da vez): aparece mesmo que o
 * tabuleiro que agiu tenha saído da tela, e some quando a vez muda.
 */
export function VetoError() {
  const { error } = useLiveMatch();
  return error ? <ErrorText>{error}</ErrorText> : null;
}
