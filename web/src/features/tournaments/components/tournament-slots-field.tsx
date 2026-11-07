import { useTranslations } from "next-intl";
import { memo } from "react";

import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { NumberInput } from "@/shared/ui/number-input";
import type { FieldErrors } from "../lib/form-errors";
import { isTdm, type TournamentValues, teamSizeOf } from "../lib/form-values";
import { SLOTS, slotLimits, teamsIn } from "../lib/format";
import { sameValues } from "../lib/same-values";

/** Os campos do formulário que o bloco lê. */
const KEYS = ["mode", "teamSize", "stages", "maxParticipants"] as const;

type Props = {
  values: Pick<TournamentValues, (typeof KEYS)[number]>;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** O torneio já começou: só para leitura. */
  readOnly: boolean;
};

/**
 * As vagas, em jogadores (a inscrição é individual), na seção Formato: os
 * limites seguem o formato da primeira fase e o time, de dois times até o teto do formato
 * vezes o tamanho, e as rodadas do suíço seguem as vagas. No TDM, a dica diz
 * quantos times cabem e quantos sobram de reserva.
 */
export const TournamentSlotsField = memo(function TournamentSlotsField({
  values,
  onChange,
  errors,
  readOnly,
}: Props) {
  const t = useTranslations("TournamentForm");
  const errorMessage = useApiErrorMessage();
  const tdm = isTdm(values);
  const size = teamSizeOf(values);
  const limits = slotLimits(values.stages[0].format, size);
  const slots = values.maxParticipants;
  const error = errors.maxParticipants;
  // No duelo, "Vagas" se explica sozinho.
  const hint =
    tdm && slots !== null
      ? t("slotsTeams", { slots, size, ...teamsIn(slots, size) })
      : undefined;

  return (
    <NumberInput
      labelText={t(tdm ? "maxParticipantsTdm" : "maxParticipants")}
      helperText={hint}
      name="maxParticipants"
      required={!readOnly}
      allowEmpty
      disableWheel
      min={limits.min}
      max={limits.max}
      value={slots}
      readOnly={readOnly}
      invalid={error !== undefined}
      // Fora dos limites o próprio campo fica inválido, com o texto da API.
      invalidText={
        error ??
        (limits.capped && (slots ?? 0) > limits.max
          ? errorMessage("format.too_many_participants", { max: limits.max })
          : errorMessage("tournament.max_participants_invalid", {
              min: limits.min,
              max: SLOTS.max,
            }))
      }
      onChange={(maxParticipants) => onChange({ maxParticipants })}
    />
  );
}, sameValues(KEYS));
