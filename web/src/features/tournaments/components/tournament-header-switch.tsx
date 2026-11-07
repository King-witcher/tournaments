"use client";

import type { ReactNode } from "react";

import { useTournamentSection } from "../hooks/use-tournament-section";

type Props = {
  /** O cabeçalho completo, na Visão geral. */
  overview: ReactNode;
  /** O cabeçalho das outras seções, com o nome da seção no título. */
  section: ReactNode;
};

/**
 * Escolhe o cabeçalho do torneio pela seção aberta. Fica no `layout.tsx` das
 * seções (os dois vêm prontos do servidor), para trocar no clique, sem
 * esperar a seção chegar.
 */
export function TournamentHeaderSwitch({ overview, section }: Props) {
  return useTournamentSection() === "overview" ? overview : section;
}
