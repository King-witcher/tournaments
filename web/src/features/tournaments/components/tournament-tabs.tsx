import { useTranslations } from "next-intl";
import type { ReactNode } from "react";
import { startedStages } from "@/features/stages/lib/stages";
import type { components } from "@/shared/api/schema";
import { isTable } from "../lib/format";
import { hasStarted, isFinished } from "../lib/status";
import { tournamentTabs } from "../lib/tabs";
import { TournamentNavRegistration } from "./tournament-nav-registration";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
  /** Organizer ou owner: vê Configurações. */
  organizer: boolean;
  /** A seção atual. */
  children: ReactNode;
};

/**
 * As seções do torneio (`tournamentTabs`) ficam no menu lateral da
 * comunidade; daqui vão as exatas para ele (a Classificação, com uma fase de
 * todos contra todos ou suíço começada, existe a partir do início; senão, só
 * no fim), e a seção atual vem logo embaixo do cabeçalho.
 */
export function TournamentTabs({
  tenant,
  tournament,
  organizer,
  children,
}: Props) {
  const t = useTranslations("Tournament");
  const standings = startedStages(tournament).some(isTable)
    ? hasStarted(tournament.status)
    : isFinished(tournament.status);
  const tabs = tournamentTabs(`/${tenant}/${tournament.slug}`, {
    status: tournament.status,
    standings,
    organizer,
  }).map((tab) => ({ href: tab.href, label: t(`tabs.${tab.key}`) }));

  return (
    <>
      <TournamentNavRegistration slug={tournament.slug} tabs={tabs} />
      {children}
    </>
  );
}
