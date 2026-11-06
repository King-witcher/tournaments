"use client";

import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { useSettingsSection } from "../hooks/use-settings-section";
import { useTournamentSettings } from "../hooks/use-tournament-settings";
import { useValuesSummary } from "../hooks/use-values-summary";
import { sectionLock } from "../lib/settings-sections";
import { SettingsSection } from "./settings-section";
import { TournamentVetoFields } from "./tournament-veto-fields";

type Props = {
  /** Os mapas ativos, para o pool. */
  maps: components["schemas"]["Map"][];
};

/**
 * Mapas: como sai o mapa de cada partida, a ordem do veto, o pool (lembrado
 * por comunidade neste navegador) e o tempo de cada passo. Só para leitura
 * depois de começar, só com os mapas do pool.
 */
export function SettingsMapsSection({ maps }: Props) {
  const tSettings = useTranslations("TournamentSettings");
  const { tenant, tournament } = useTournamentSettings();
  const form = useSettingsSection("maps");
  const summary = useValuesSummary();
  const lock = sectionLock("maps", tournament.status);

  return (
    <SettingsSection
      section="maps"
      form={form}
      summary={summary.veto(form.saved, maps)}
      lock={lock && tSettings(`locks.${lock}`)}
      readOnly={lock !== undefined}
      onSave={form.submit}
    >
      <TournamentVetoFields
        tenant={tenant}
        values={form.values}
        onChange={form.onChange}
        errors={form.errors}
        maps={maps}
        readOnly={lock !== undefined}
      />
    </SettingsSection>
  );
}
