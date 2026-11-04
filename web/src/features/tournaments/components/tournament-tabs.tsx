import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { PageTabs } from "@/shared/ui/page-tabs";
import { isTable } from "../lib/format";
import { hasStarted, isFinished } from "../lib/status";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
  /** Organizer ou owner: vê a aba Organização. */
  organizer: boolean;
  /** A aba atual. */
  children: ReactNode;
};

/**
 * As abas do torneio, uma coisa por aba: Visão geral, Chave (a partir do
 * início), Classificação (no round robin e no suíço, a partir do início; na
 * eliminação, só no fim, quando há colocações), Inscritos, Regras (sempre: o
 * texto que sai do formato vem antes das regras adicionais) e Organização (só
 * para ela). O que não se aplica não entra na lista.
 */
export function TournamentTabs({
  tenant,
  tournament,
  organizer,
  children,
}: Props) {
  const t = useTranslations("Tournament");
  const base = `/${tenant}/${tournament.slug}`;
  const standings = isTable(tournament.stages[0])
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
    ...(organizer ? [{ href: `${base}/manage`, label: t("tabs.manage") }] : []),
  ];

  return (
    <PageTabs label={t("sections")} tabs={tabs}>
      {children}
    </PageTabs>
  );
}
