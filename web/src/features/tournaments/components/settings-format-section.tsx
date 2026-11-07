"use client";

import { useTranslations } from "next-intl";
import { useCallback } from "react";

import { useSettingsDrafts } from "../hooks/use-settings-drafts";
import { useSettingsSection } from "../hooks/use-settings-section";
import { useTournamentSettings } from "../hooks/use-tournament-settings";
import { useValuesSummary } from "../hooks/use-values-summary";
import { isTdm } from "../lib/form-values";
import { sectionLock } from "../lib/settings-sections";
import { stageStarted } from "../lib/stage-values";
import { SettingsSection } from "./settings-section";
import { TournamentModeFields } from "./tournament-mode-fields";
import { TournamentRatingField } from "./tournament-rating-field";
import { TournamentSlotsField } from "./tournament-slots-field";
import { TournamentStages } from "./tournament-stages";
import { TournamentTeamRulesFields } from "./tournament-team-rules-fields";

/**
 * Formato: modalidade, tamanho do time e vagas numa linha (com o aviso do que
 * salvar faz com os times dos inscritos) e, no TDM, quem age pelo time logo
 * embaixo, assim que a modalidade muda na tela; as fases (cada uma com
 * chaveamento, séries, final e rodadas, reset e, nas de tabela, o Desempate;
 * da segunda em diante, quem entra e o check-in) e o rating. As séries que
 * pedem mais mapas que o pool salvo em Mapas ficam inválidas na hora, com o
 * caminho para a página de Mapas.
 * Depois de começar, os campos do torneio e as fases que começaram ficam só
 * para leitura; as que não começaram ainda mudam.
 */
export function SettingsFormatSection() {
  const tSettings = useTranslations("TournamentSettings");
  const summary = useValuesSummary();
  const { tournament } = useTournamentSettings();
  const { goTo } = useSettingsDrafts();
  const form = useSettingsSection("format");
  const { values, onChange, errors, saved } = form;
  const lock = sectionLock("format", tournament.status);
  // Os campos do torneio param no início; as fases, uma a uma.
  const fixed = lock !== undefined;
  const stagesOpen =
    lock !== "finished" && !tournament.stages.every(stageStarted);
  const readOnly = fixed && !stagesOpen;
  // O pool e o modo do mapa vêm do salvo em Mapas.
  const pool =
    !fixed && values.mapMode !== "off" ? values.mapPool?.length : undefined;
  const goToMaps = useCallback(() => goTo("maps"), [goTo]);

  return (
    <SettingsSection
      section="format"
      form={form}
      summary={[summary.format(saved), isTdm(saved) && summary.teamRules(saved)]
        .filter(Boolean)
        .join(" · ")}
      lock={
        lock &&
        tSettings(
          lock === "started" && stagesOpen
            ? "locks.startedStages"
            : `locks.${lock}`,
        )
      }
      readOnly={readOnly}
      onSave={form.submit}
    >
      <TournamentModeFields
        values={values}
        onChange={onChange}
        errors={errors}
        readOnly={fixed}
        saved={tournament}
      >
        <TournamentSlotsField
          values={values}
          onChange={onChange}
          errors={errors}
          readOnly={fixed}
        />
      </TournamentModeFields>
      {isTdm(values) && (
        <TournamentTeamRulesFields
          values={values}
          onChange={onChange}
          readOnly={fixed}
        />
      )}
      <TournamentStages
        values={values}
        onChange={onChange}
        errors={errors}
        pool={pool}
        onGoToPool={goToMaps}
      />
      <TournamentRatingField
        values={values}
        onChange={onChange}
        readOnly={fixed}
      />
    </SettingsSection>
  );
}
