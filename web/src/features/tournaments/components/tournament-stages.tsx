"use client";

import { useTranslations } from "next-intl";
import { useCallback, useEffect, useRef, useState } from "react";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button } from "@/shared/ui/button";
import { Stack } from "@/shared/ui/carbon/stack";
import { Tab, TabPanel, TabPanels, Tabs } from "@/shared/ui/carbon/tabs";
import { FormSection } from "@/shared/ui/form-section";
import { TabList } from "@/shared/ui/tab-list";
import { useTournamentSettings } from "../hooks/use-tournament-settings";
import {
  type FieldErrors,
  isStageField,
  stageIndexOf,
} from "../lib/form-errors";
import type { TournamentValues } from "../lib/form-values";
import {
  addedStage,
  MAX_STAGES,
  type StageValues,
  stageStarted,
} from "../lib/stage-values";
import { isFinished, setupEditable } from "../lib/status";
import { isTableFormat } from "../lib/tiebreakers";
import { TournamentFormatFields } from "./tournament-format-fields";
import { TournamentStageEntryFields } from "./tournament-stage-entry-fields";
import styles from "./tournament-stages.module.scss";
import { TournamentTiebreakerFields } from "./tournament-tiebreaker-fields";

type Props = {
  values: Pick<
    TournamentValues,
    "stages" | "mode" | "teamSize" | "maxParticipants"
  >;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** Os mapas do pool salvo, quando o mapa sai dele (ver `TournamentFormatFields`). */
  pool?: number;
  /** Leva a Mapas, onde o pool cresce (a mesma função em todo render). */
  onGoToPool?: () => void;
};

/**
 * As fases do torneio. Com uma, os campos dela direto, e "Adicionar fase"
 * embaixo; com duas a quatro, uma aba por fase, cada uma com os campos, o
 * Desempate, a entrada (da segunda em diante) e "Remover fase". Antes do
 * início tudo muda; depois, só as fases que não começaram (as que começaram
 * ficam só para leitura), e não se acrescenta fase. A eliminação só cabe na
 * última fase, então não se acrescenta fase depois dela. Um erro da API numa
 * fase abre a aba dela.
 */
export function TournamentStages({
  values,
  onChange,
  errors,
  pool,
  onGoToPool,
}: Props) {
  const t = useTranslations("TournamentForm");
  const { tournament } = useTournamentSettings();
  const { stages } = values;
  const [active, setActive] = useState(0);
  const current = Math.min(active, stages.length - 1);

  // As alterações partem das fases de agora, sem a função mudar a cada uma.
  const latest = useRef(stages);
  latest.current = stages;
  const onStageChange = useCallback(
    (index: number, patch: Partial<StageValues>) =>
      onChange({
        stages: latest.current.map((s, i) =>
          i === index ? { ...s, ...patch } : s,
        ),
      }),
    [onChange],
  );

  // A aba da fase que a API recusou.
  useEffect(() => {
    const field = Object.keys(errors).find(isStageField);
    if (field) setActive(stageIndexOf(field));
  }, [errors]);

  const finished = isFinished(tournament.status);
  const locked = (index: number) =>
    finished || stageStarted(tournament.stages[index]);
  const canAdd = setupEditable(tournament.status) && stages.length < MAX_STAGES;
  const eliminationLast = stages[stages.length - 1].format === "elimination";

  const add = () => {
    onChange({ stages: [...stages, addedStage()] });
    setActive(stages.length);
  };
  const remove = (index: number) => {
    const next = stages.filter((_, i) => i !== index);
    // A primeira fase não tem corte nem check-in.
    if (index === 0) next[0] = { ...next[0], advance: null, checkIn: false };
    onChange({ stages: next });
    setActive(Math.max(0, index - 1));
  };

  const fields = (stage: StageValues, index: number) => (
    <Stack gap={6}>
      <TournamentFormatFields
        values={values}
        stage={stage}
        index={index}
        last={index === stages.length - 1}
        onStageChange={onStageChange}
        errors={errors}
        readOnly={locked(index)}
        pool={locked(index) ? undefined : pool}
        onGoToPool={onGoToPool}
      />
      {isTableFormat(stage.format) && (
        <FormSection legend={t("tiebreakers.title")}>
          <TournamentTiebreakerFields
            stage={stage}
            index={index}
            onStageChange={onStageChange}
            errors={errors}
            readOnly={locked(index)}
          />
        </FormSection>
      )}
      {index > 0 && (
        <FormSection legend={t("stages.entryTitle")}>
          <TournamentStageEntryFields
            stage={stage}
            index={index}
            onStageChange={onStageChange}
            errors={errors}
            readOnly={locked(index)}
          />
        </FormSection>
      )}
      {stages.length > 1 && !locked(index) && (
        <div>
          <Button kind="danger--ghost" size="sm" onClick={() => remove(index)}>
            {t("stages.remove")}
          </Button>
        </div>
      )}
    </Stack>
  );

  return (
    <Stack gap={6}>
      {stages.length === 1 ? (
        fields(stages[0], 0)
      ) : (
        // As abas de linha: dentro do acordeão, sem área própria para o
        // painel (a `contained` do Carbon é para conteúdo num contêiner). O
        // `div` faz das abas e do painel um item só do `Stack`.
        <div>
          <Tabs
            selectedIndex={current}
            onChange={({ selectedIndex }) => setActive(selectedIndex)}
          >
            <TabList aria-label={t("stages.label")}>
              {stages.map((_, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: as fases são a posição
                <Tab key={i}>{t("stages.tab", { number: i + 1 })}</Tab>
              ))}
            </TabList>
            <TabPanels>
              {stages.map((stage, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: as fases são a posição
                <TabPanel key={i} className={styles.panel}>
                  {fields(stage, i)}
                </TabPanel>
              ))}
            </TabPanels>
          </Tabs>
        </div>
      )}
      {canAdd && (
        <ActionWithNote
          note={eliminationLast ? t("stages.addBlocked") : undefined}
        >
          <Button kind="tertiary" disabled={eliminationLast} onClick={add}>
            {t("stages.add")}
          </Button>
        </ActionWithNote>
      )}
    </Stack>
  );
}
