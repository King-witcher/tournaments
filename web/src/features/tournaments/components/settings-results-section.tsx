"use client";

import { useTranslations } from "next-intl";

import { useSettingsSection } from "../hooks/use-settings-section";
import { useTournamentSettings } from "../hooks/use-tournament-settings";
import { useValuesSummary } from "../hooks/use-values-summary";
import { sectionLock } from "../lib/settings-sections";
import { SettingsSection } from "./settings-section";
import { TournamentResultsFields } from "./tournament-results-fields";

type Props = {
  /** Se esta API tem capturas de tela. */
  screenshots: boolean;
};

/**
 * Resultados: a janela de contestação e a captura de tela obrigatória (com
 * capturas na API). Mudam até o torneio encerrar.
 */
export function SettingsResultsSection({ screenshots }: Props) {
  const tSettings = useTranslations("TournamentSettings");
  const { tournament } = useTournamentSettings();
  const form = useSettingsSection("results");
  const summary = useValuesSummary();
  const lock = sectionLock("results", tournament.status);

  return (
    <SettingsSection
      section="results"
      form={form}
      summary={summary.results(form.saved, screenshots)}
      lock={lock && tSettings(`locks.${lock}`)}
      readOnly={lock !== undefined}
      onSave={form.submit}
      reading
    >
      <TournamentResultsFields
        values={form.values}
        onChange={form.onChange}
        errors={form.errors}
        readOnly={lock !== undefined}
        screenshots={screenshots}
      />
    </SettingsSection>
  );
}
