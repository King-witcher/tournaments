import { useTranslations } from "next-intl";
import { memo, useState } from "react";

import { Checkbox } from "@/shared/ui/checkbox";
import { FormRow } from "@/shared/ui/form-row";
import { NumberInput } from "@/shared/ui/number-input";
import { RadioGroup } from "@/shared/ui/radio-group";
import { type FieldErrors, stageField } from "../lib/form-errors";
import { slotLimits } from "../lib/format";
import type { StageValues } from "../lib/stage-values";

/** Quantos entram por padrão ao escolher "Os melhores". */
const DEFAULT_ADVANCE = 8;

type Props = {
  stage: Pick<StageValues, "format" | "advance" | "checkIn">;
  /** A posição da fase, a partir de 0 (de 1 em diante: a primeira não tem entrada). */
  index: number;
  onStageChange: (index: number, patch: Partial<StageValues>) => void;
  errors: FieldErrors;
  /** A fase não muda: tudo aqui só para leitura. */
  readOnly: boolean;
};

/**
 * Como a fase recebe a anterior, da segunda em diante: todos ou os N melhores
 * da classificação dela (de 2 ao teto do formato desta fase), com o número ao
 * lado da escolha, e o check-in antes da fase, que vale ao salvar.
 */
export const TournamentStageEntryFields = memo(
  function TournamentStageEntryFields({
    stage,
    index,
    onStageChange,
    errors,
    readOnly,
  }: Props) {
    const t = useTranslations("TournamentForm.stages");
    const advanceName = stageField(index, "advance");
    const max = slotLimits(stage.format, 1).max;
    const error = errors[advanceName];
    // "Os melhores" com o campo vazio continua escolhido (o obrigatório avisa).
    const [top, setTop] = useState(stage.advance !== null);
    const limited = top || stage.advance !== null;

    return (
      <>
        <FormRow columns={2}>
          <RadioGroup
            legendText={t("entry", { previous: index })}
            name={`stages.${index}.entry`}
            orientation="horizontal"
            value={limited ? "top" : "all"}
            readOnly={readOnly}
            options={(["all", "top"] as const).map((value) => ({
              value,
              label: t(`entryOptions.${value}`),
            }))}
            onChange={(entry) => {
              setTop(entry === "top");
              onStageChange(index, {
                advance:
                  entry === "all" ? null : Math.min(DEFAULT_ADVANCE, max),
              });
            }}
          />
          {limited && (
            <NumberInput
              labelText={t("entryCount")}
              name={advanceName}
              required={!readOnly}
              disableWheel
              min={2}
              max={max}
              value={stage.advance}
              readOnly={readOnly}
              invalid={error !== undefined}
              invalidText={error}
              onChange={(advance) => onStageChange(index, { advance })}
            />
          )}
        </FormRow>
        <Checkbox
          labelText={t("checkIn")}
          helperText={t("checkInHint")}
          name={`stages.${index}.checkIn`}
          checked={stage.checkIn}
          readOnly={readOnly}
          onChange={(checkIn) => onStageChange(index, { checkIn })}
        />
      </>
    );
  },
);
