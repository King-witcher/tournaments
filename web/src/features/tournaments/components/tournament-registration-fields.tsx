import { useTranslations } from "next-intl";

import { ScreenshotsRequiredField } from "@/features/screenshots/components/screenshots-required-field";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { NumberInput } from "@/shared/ui/number-input";
import { RadioGroup } from "@/shared/ui/radio-group";
import type { FieldErrors } from "../lib/form-errors";
import type { TournamentValues } from "../lib/form-values";
import { TournamentCheckInField } from "./tournament-check-in-field";
import { TournamentContestField } from "./tournament-contest-field";

/** Os limites de vagas da API. */
const SLOTS = { min: 2, max: 1024 };

type Props = {
  values: TournamentValues;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** O torneio já começou: vagas e check-in ficam travados. */
  setupLocked: boolean;
  /** Encerrado: nem a visibilidade, nem a janela de contestação, nem o screenshot obrigatório mudam. */
  finished: boolean;
};

/**
 * Inscrições e prazos: vagas, visibilidade, janela de check-in, janela de
 * contestação e screenshot obrigatório (só com screenshots na API). Todos têm
 * padrão; a data de início fica fora, junto do nome.
 */
export function TournamentRegistrationFields({
  values,
  onChange,
  errors,
  setupLocked,
  finished,
}: Props) {
  const t = useTranslations("TournamentForm");
  const errorMessage = useApiErrorMessage();

  return (
    <>
      <NumberInput
        labelText={t("maxParticipants")}
        helperText={t("maxParticipantsHint")}
        name="maxParticipants"
        required
        allowEmpty
        disableWheel
        min={SLOTS.min}
        max={SLOTS.max}
        value={values.maxParticipants}
        disabled={setupLocked}
        invalid={errors.maxParticipants !== undefined}
        // Fora dos limites o próprio campo fica inválido, com o texto da API.
        invalidText={
          errors.maxParticipants ??
          errorMessage("tournament.max_participants_invalid", SLOTS)
        }
        onChange={(maxParticipants) => onChange({ maxParticipants })}
      />
      <RadioGroup
        legendText={t("visibility")}
        helperText={t(`visibilityHints.${values.visibility}`)}
        name="visibility"
        orientation="horizontal"
        value={values.visibility}
        disabled={finished}
        options={[
          { value: "public", label: t("visibilityOptions.public") },
          { value: "private", label: t("visibilityOptions.private") },
        ]}
        onChange={(visibility) => onChange({ visibility })}
      />
      <TournamentCheckInField
        value={values.checkInMinutes}
        onChange={(checkInMinutes) => onChange({ checkInMinutes })}
        error={errors.checkInMinutes}
        disabled={setupLocked}
      />
      <TournamentContestField
        value={values.contestMinutes}
        onChange={(contestMinutes) => onChange({ contestMinutes })}
        error={errors.contestMinutes}
        disabled={finished}
      />
      <ScreenshotsRequiredField
        checked={values.screenshotsRequired}
        onChange={(screenshotsRequired) => onChange({ screenshotsRequired })}
        disabled={finished}
      />
    </>
  );
}
