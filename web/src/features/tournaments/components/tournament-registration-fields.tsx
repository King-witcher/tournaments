import { useTranslations } from "next-intl";

import { ScreenshotsRequiredField } from "@/features/screenshots/components/screenshots-required-field";
import { RadioGroup } from "@/shared/ui/radio-group";
import type { FieldErrors } from "../lib/form-errors";
import { isTdm, type TournamentValues } from "../lib/form-values";
import { TournamentCheckInField } from "./tournament-check-in-field";
import { TournamentContestField } from "./tournament-contest-field";
import { TournamentSlotsField } from "./tournament-slots-field";

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
