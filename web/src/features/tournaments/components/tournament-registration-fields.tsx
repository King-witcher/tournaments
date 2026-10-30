import { useTranslations } from "next-intl";

import { RadioGroup } from "@/shared/ui/radio-group";
import type { FieldErrors } from "../lib/form-errors";
import { isTdm, type TournamentValues } from "../lib/form-values";
import { TournamentCheckInField } from "./tournament-check-in-field";
import { TournamentSlotsField } from "./tournament-slots-field";

type Props = {
  values: TournamentValues;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** O torneio já começou: vagas e check-in ficam travados. */
  setupLocked: boolean;
  /** Encerrado: a visibilidade não muda mais. */
  finished: boolean;
};

/**
 * Inscrições e prazos: vagas, visibilidade e janela de check-in, todos com
 * padrão. A data de início fica fora, junto do nome; a contestação e a captura
 * de tela, em Resultados.
 */
export function TournamentRegistrationFields({
  values,
  onChange,
  errors,
  setupLocked,
  finished,
}: Props) {
  const t = useTranslations("TournamentForm");

  return (
    <>
      <TournamentSlotsField
        values={values}
        onChange={(maxParticipants) => onChange({ maxParticipants })}
        error={errors.maxParticipants}
        disabled={setupLocked}
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
        teams={isTdm(values)}
      />
    </>
  );
}
