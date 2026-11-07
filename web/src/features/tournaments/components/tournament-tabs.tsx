import { useTranslations } from "next-intl";
import type { ReactNode } from "react";
import { startedStages } from "@/features/stages/lib/stages";
import type { components } from "@/shared/api/schema";
import { PageTabs } from "@/shared/ui/page-tabs";
import { isTable } from "../lib/format";
import { hasStarted, isFinished } from "../lib/status";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
  /** Organizer ou owner: vê as abas Organização e Configurações. */
  organizer: boolean;
  /** A aba atual. */
  children: ReactNode;
};

/**
 * As abas do torneio, uma coisa por aba: Visão geral, Chave (a partir do
 * início), Classificação (com uma fase de todos contra todos ou suíço começada, a
 * partir do início; senão, só no fim, quando há colocações), Inscritos, Regras (sempre: o
 * texto que sai do formato vem antes das regras da organização), Organização e
 * Configurações (só para ela). O que não se aplica não entra na lista.
 */
export function TournamentTabs({
  tenant,
  tournament,
  organizer,
  children,
}: Props) {
  const t = useTranslations("Tournament");
  const base = `/${tenant}/${tournament.slug}`;
  // Alguma fase começada com tabela dá a Classificação desde o início; só com
  // eliminação (e a disputa), ela existe quando o torneio encerra.
  const standings = startedStages(tournament).some(isTable)
    ? hasStarted(tournament.status)
    : isFinished(tournament.status);
  const tabs = [
    { href: base, label: t("tabs.overview") },
    ...(hasStarted(tournament.status)
      ? [{ href: `${base}/bracket`, label: t("tabs.bracket") }]
      : []),
    ...(standings
      ? [{ href: `${base}/standings`, label: t("tabs.standings") }]
      : []),
    { href: `${base}/participants`, label: t("tabs.participants") },
    { href: `${base}/rules`, label: t("tabs.rules") },
    ...(organizer
      ? [
          { href: `${base}/manage`, label: t("tabs.manage") },
          { href: `${base}/settings`, label: t("tabs.settings") },
        ]
      : []),
  ];

  return (
    <PageTabs label={t("sections")} tabs={tabs}>
      {children}
    </PageTabs>
  );
}
