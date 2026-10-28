import { useTranslations } from "next-intl";

import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useIsClient } from "@/shared/hooks/use-is-client";
import { browserTimeZone } from "@/shared/lib/datetime-local";
import { DateTimeInput } from "@/shared/ui/date-time-input";
import { NumberInput } from "@/shared/ui/number-input";
import type { FieldErrors } from "../lib/form-errors";
import type { TournamentValues } from "../lib/form-values";
import { TournamentCheckInField } from "./tournament-check-in-field";
import { TournamentContestField } from "./tournament-contest-field";
import { TournamentFormGroup } from "./tournament-form-group";

/** Os limites de vagas da API. */
const SLOTS = { min: 2, max: 1024 };

type Props = {
  values: TournamentValues;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** O torneio já começou: vagas, data e check-in ficam travados. */
  setupLocked: boolean;
  /** Publicado: a data não pode ficar vazia. */
  published: boolean;
  /** Encerrado: nem a janela de contestação muda. */
  finished: boolean;
};

/**
 * Inscrições e datas: vagas, início (no fuso de quem preenche), janela de
 * check-in e janela de contestação.
 */
export function TournamentScheduleFields({
  values,
  onChange,
  errors,
  setupLocked,
  published,
  finished,
}: Props) {
  const t = useTranslations("TournamentForm");
  const errorMessage = useApiErrorMessage();
  const isClient = useIsClient();

  return (
    <TournamentFormGroup legend={t("schedule")}>
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
      <DateTimeInput
        labelText={t("startsAt")}
        helperText={t("startsAtHint", {
          timeZone: isClient ? browserTimeZone() : "…",
        })}
        name="startsAt"
        value={values.startsAt}
        required={published}
        disabled={setupLocked}
        invalid={errors.startsAt !== undefined}
        invalidText={errors.startsAt}
        onChange={(startsAt) => onChange({ startsAt })}
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
    </TournamentFormGroup>
  );
}
