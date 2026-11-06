"use client";

import { useTranslations } from "next-intl";

import { useSettingsSection } from "../hooks/use-settings-section";
import { useTournamentSettings } from "../hooks/use-tournament-settings";
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
  const lock = sectionLock("results", tournament.status);

  return (
    <SettingsSection
      section="results"
      form={form}
      lock={lock && tSettings(`locks.${lock}`)}
      readOnly={lock !== undefined}
      onSave={form.submit}
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
