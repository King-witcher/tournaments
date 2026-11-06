"use client";

import { useTranslations } from "next-intl";
import { useCallback } from "react";

import { FormSection } from "@/shared/ui/form-section";
import { useSettingsItems } from "../hooks/use-settings-items";
import { useSettingsSection } from "../hooks/use-settings-section";
import { useTournamentSettings } from "../hooks/use-tournament-settings";
import { useValuesSummary } from "../hooks/use-values-summary";
import { isTdm } from "../lib/form-values";
import { sectionLock } from "../lib/settings-sections";
import { isTableFormat } from "../lib/tiebreakers";
import { SettingsSection } from "./settings-section";
import { TournamentFormatFields } from "./tournament-format-fields";
import { TournamentModeFields } from "./tournament-mode-fields";
import { TournamentSlotsField } from "./tournament-slots-field";
import { TournamentTeamRulesFields } from "./tournament-team-rules-fields";
import { TournamentTiebreakerFields } from "./tournament-tiebreaker-fields";

/**
 * Formato: modalidade e tamanho do time (com o aviso do que salvar faz com os
 * times dos inscritos) e, no TDM, quem age pelo time logo embaixo, assim que
 * a modalidade muda na tela; vagas, chaveamento, séries, final, reset,
 * rodadas, rating e, nas fases de tabela, o Desempate, que completa o
 * formato. As séries que pedem mais mapas que o pool salvo em Mapas ficam
 * inválidas na hora, com o caminho para lá. Só para leitura depois de
 * começar.
 */
export function SettingsFormatSection() {
  const t = useTranslations("TournamentForm");
  const tSettings = useTranslations("TournamentSettings");
  const summary = useValuesSummary();
  const { tournament } = useTournamentSettings();
  const { reveal } = useSettingsItems();
  const form = useSettingsSection("format");
  const { values, onChange, errors, saved } = form;
  const lock = sectionLock("format", tournament.status);
  const readOnly = lock !== undefined;
  // O pool e o modo do mapa vêm do salvo em Mapas.
  const pool =
    !readOnly && values.mapMode !== "off" ? values.mapPool?.length : undefined;
  const goToMaps = useCallback(() => reveal("maps"), [reveal]);

  return (
    <SettingsSection
      section="format"
      form={form}
      summary={[summary.format(saved), isTdm(saved) && summary.teamRules(saved)]
        .filter(Boolean)
        .join(" · ")}
      lock={lock && tSettings(`locks.${lock}`)}
      readOnly={readOnly}
      onSave={form.submit}
    >
      <TournamentModeFields
        values={values}
        onChange={onChange}
        errors={errors}
        readOnly={readOnly}
        saved={tournament}
      />
      {isTdm(values) && (
        <TournamentTeamRulesFields
          values={values}
          onChange={onChange}
          readOnly={readOnly}
        />
      )}
      <TournamentSlotsField
        values={values}
        onChange={onChange}
        errors={errors}
        readOnly={readOnly}
      />
      <TournamentFormatFields
        values={values}
        onChange={onChange}
        errors={errors}
        readOnly={readOnly}
        pool={pool}
        onGoToPool={goToMaps}
      />
      {isTableFormat(values.stageFormat) && (
        <FormSection legend={t("tiebreakers.title")}>
          <TournamentTiebreakerFields
            values={values}
            onChange={onChange}
            errors={errors}
            readOnly={readOnly}
          />
        </FormSection>
      )}
    </SettingsSection>
  );
}
