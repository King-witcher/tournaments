"use client";

import { useTranslations } from "next-intl";

import { useSettingsSection } from "../hooks/use-settings-section";
import { useTournamentSettings } from "../hooks/use-tournament-settings";
import { sectionLock } from "../lib/settings-sections";
import { SettingsSection } from "./settings-section";
import { TournamentTeamRulesFields } from "./tournament-team-rules-fields";

/**
 * Times (só no TDM): quem age pelo time. As regras que não têm efeito com os
 * mapas e campeões salvos não aparecem. Só para leitura depois de começar.
 */
export function SettingsTeamsSection() {
  const tSettings = useTranslations("TournamentSettings");
  const { tournament } = useTournamentSettings();
  const form = useSettingsSection("teams");
  const lock = sectionLock("teams", tournament.status);

  return (
    <SettingsSection
      section="teams"
      form={form}
      lock={lock && tSettings(`locks.${lock}`)}
      readOnly={lock !== undefined}
      onSave={form.submit}
    >
      <TournamentTeamRulesFields
        values={form.values}
        onChange={form.onChange}
        readOnly={lock !== undefined}
      />
    </SettingsSection>
  );
}
