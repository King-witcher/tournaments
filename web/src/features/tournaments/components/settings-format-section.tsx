"use client";

import { useTranslations } from "next-intl";

import { FormSection } from "@/shared/ui/form-section";
import { useSettingsSection } from "../hooks/use-settings-section";
import { useTournamentSettings } from "../hooks/use-tournament-settings";
import { sectionLock } from "../lib/settings-sections";
import { isTableFormat } from "../lib/tiebreakers";
import { SettingsSection } from "./settings-section";
import { TournamentFormatFields } from "./tournament-format-fields";
import { TournamentModeFields } from "./tournament-mode-fields";
import { TournamentSlotsField } from "./tournament-slots-field";
import { TournamentTiebreakerFields } from "./tournament-tiebreaker-fields";

/**
 * Formato: modalidade e tamanho do time (com o aviso do que salvar faz com os
 * times dos inscritos), vagas, chaveamento, séries, final, reset, rodadas,
 * rating e, nas fases de tabela, o Desempate, que completa o formato. Só para
 * leitura depois de começar.
 */
export function SettingsFormatSection() {
  const t = useTranslations("TournamentForm");
  const tSettings = useTranslations("TournamentSettings");
  const { tournament } = useTournamentSettings();
  const form = useSettingsSection("format");
  const { values, onChange, errors } = form;
  const lock = sectionLock("format", tournament.status);
  const readOnly = lock !== undefined;

  return (
    <SettingsSection
      section="format"
      form={form}
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
