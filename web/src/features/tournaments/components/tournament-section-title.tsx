"use client";

import { useTranslations } from "next-intl";

import { useTournamentSection } from "../hooks/use-tournament-section";

/** O nome da seção aberta, no título do cabeçalho compacto do torneio. */
export function TournamentSectionTitle() {
  const t = useTranslations("Tournament.tabs");
  return t(useTournamentSection());
}
