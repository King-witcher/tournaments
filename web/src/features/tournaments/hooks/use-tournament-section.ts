"use client";

import { useSelectedLayoutSegment } from "next/navigation";

import type { TournamentTabKey } from "../lib/tabs";

const SECTIONS: readonly string[] = [
  "bracket",
  "standings",
  "participants",
  "rules",
  "manage",
  "settings",
] satisfies TournamentTabKey[];

/**
 * A seção do torneio aberta, lida do segmento abaixo do `layout.tsx` das
 * seções: a Visão geral é a página do próprio grupo (sem segmento, ou o
 * `__PAGE__` do Next). Só no cliente, dentro da casca do torneio.
 */
export function useTournamentSection(): TournamentTabKey {
  const segment = useSelectedLayoutSegment();
  return segment && SECTIONS.includes(segment)
    ? (segment as TournamentTabKey)
    : "overview";
}
