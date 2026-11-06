"use client";

import { useTranslations } from "next-intl";

import { useSettingsSection } from "../hooks/use-settings-section";
import { useTournamentSettings } from "../hooks/use-tournament-settings";
import { useValuesSummary } from "../hooks/use-values-summary";
import { sectionLock } from "../lib/settings-sections";
import { SettingsSection } from "./settings-section";
import { TournamentChampionFields } from "./tournament-champion-fields";

/**
 * Campeões: o draft da série, os bans, a ordem das escolhas, o que pode
 * repetir e os tempos que não ficam em Mapas. Só para leitura depois de
 * começar.
 */
export function SettingsChampionsSection() {
  const tSettings = useTranslations("TournamentSettings");
  const { tournament } = useTournamentSettings();
  const form = useSettingsSection("champions");
  const summary = useValuesSummary();
  const lock = sectionLock("champions", tournament.status);

  return (
    <SettingsSection
      section="champions"
      form={form}
      summary={summary.champions(form.saved)}
      lock={lock && tSettings(`locks.${lock}`)}
      readOnly={lock !== undefined}
      onSave={form.submit}
    >
      <TournamentChampionFields
        values={form.values}
        onChange={form.onChange}
        errors={form.errors}
        readOnly={lock !== undefined}
      />
    </SettingsSection>
  );
}
