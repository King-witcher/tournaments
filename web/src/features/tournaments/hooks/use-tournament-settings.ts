import { createContext, useContext } from "react";

import type { components } from "@/shared/api/schema";

type Tournament = components["schemas"]["Tournament"];

export type TournamentSettingsState = {
  tenant: string;
  /** O torneio como está salvo agora: o da página ou o da última resposta. */
  tournament: Tournament;
  /** Uma seção salvou: as outras montam o corpo a partir deste. */
  setSaved: (tournament: Tournament) => void;
};

export const TournamentSettingsContext =
  createContext<TournamentSettingsState | null>(null);

/** O torneio salvo da aba Configurações (`TournamentSettingsProvider`). */
export function useTournamentSettings() {
  const state = useContext(TournamentSettingsContext);
  if (!state) {
    throw new Error("useTournamentSettings fora do TournamentSettingsProvider");
  }
  return state;
}
