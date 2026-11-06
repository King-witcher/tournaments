"use client";

import { type ReactNode, useMemo, useState } from "react";

import type { components } from "@/shared/api/schema";
import { TournamentSettingsContext } from "../hooks/use-tournament-settings";

type Tournament = components["schemas"]["Tournament"];

type Props = {
  tenant: string;
  /** O torneio que a página buscou. */
  tournament: Tournament;
  children: ReactNode;
};

/**
 * O torneio salvo, para todas as seções da aba Configurações: cada uma manda o
 * que está salvo mais os campos dela. Depois de salvar, vale a resposta da API
 * até a página trazer o torneio de novo (`router.refresh` ou o tempo real),
 * para outra seção salva logo em seguida não mandar o valor antigo.
 */
export function TournamentSettingsProvider({
  tenant,
  tournament,
  children,
}: Props) {
  const [fromPage, setFromPage] = useState(tournament);
  const [saved, setSaved] = useState(tournament);
  if (fromPage !== tournament) {
    setFromPage(tournament);
    setSaved(tournament);
  }
  const value = useMemo(
    () => ({ tenant, tournament: saved, setSaved }),
    [tenant, saved],
  );
  return (
    <TournamentSettingsContext value={value}>
      {children}
    </TournamentSettingsContext>
  );
}
