import { useTranslations } from "next-intl";

import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { NumberInput } from "@/shared/ui/number-input";
import { isTdm, type TournamentValues, teamSizeOf } from "../lib/form-values";
import { SLOTS, slotLimits, teamsIn } from "../lib/format";

type Props = {
  values: TournamentValues;
  onChange: (maxParticipants: number | null) => void;
  /** O que a API disse deste campo, já traduzido. */
  error?: string;
  /** O torneio já começou: não muda mais. */
  disabled: boolean;
};

/**
 * As vagas, em jogadores (a inscrição é individual). Os limites seguem o
 * formato e o time: de dois times até o teto do formato vezes o tamanho. No
 * TDM, a dica diz quantos times cabem e quantos sobram de reserva.
 */
export function TournamentSlotsField({
  values,
  onChange,
  error,
  disabled,
}: Props) {
  const t = useTranslations("TournamentForm");
  const errorMessage = useApiErrorMessage();
  const tdm = isTdm(values);
  const size = teamSizeOf(values);
  const limits = slotLimits(values.stageFormat, size);
  const slots = values.maxParticipants;
  const hint = tdm
    ? [
        slots !== null &&
          t("slotsTeams", { slots, size, ...teamsIn(slots, size) }),
        t("maxParticipantsHintTdm"),
      ]
        .filter(Boolean)
        .join(" ")
    : t("maxParticipantsHint");

  return (
    <NumberInput
      labelText={t(tdm ? "maxParticipantsTdm" : "maxParticipants")}
      helperText={hint}
      name="maxParticipants"
      required
      allowEmpty
      disableWheel
      min={limits.min}
      max={limits.max}
      value={slots}
      disabled={disabled}
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
      onChange={onChange}
    />
  );
}
