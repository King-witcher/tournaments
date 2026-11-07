"use client";

import { useEffect } from "react";

import { type NavTab, useTournamentNav } from "./tournament-nav-context";

/**
 * Diz ao menu lateral as seções exatas do torneio aberto (a Classificação
 * depende das fases, que a lista do menu não traz). Não desenha nada.
 */
export function TournamentNavRegistration({
  slug,
  tabs,
}: {
  slug: string;
  tabs: NavTab[];
}) {
  const { setCurrent } = useTournamentNav();
  const key = JSON.stringify(tabs);
  // biome-ignore lint/correctness/useExhaustiveDependencies: `key` resume `tabs`
  useEffect(() => {
    setCurrent({ slug, tabs });
    return () => setCurrent(null);
  }, [slug, key, setCurrent]);
  return null;
}
