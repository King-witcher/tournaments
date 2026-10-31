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
  /** Encerrado: o tipo de inscrição não muda mais. */
  finished: boolean;
};

/**
 * Inscrições e prazos: vagas, tipo de inscrição e janela de check-in, todos com
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
        legendText={t("admission")}
        // "Só por link ou pela organização" já diz tudo; da aberta, falta o
        // prazo.
        helperText={
          values.admission === "open" ? t("admissionOpenHint") : undefined
        }
        name="admission"
        orientation="horizontal"
        value={values.admission}
        disabled={finished}
        options={[
          { value: "open", label: t("admissionOptions.open") },
          { value: "invite_only", label: t("admissionOptions.invite_only") },
        ]}
        onChange={(admission) => onChange({ admission })}
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
