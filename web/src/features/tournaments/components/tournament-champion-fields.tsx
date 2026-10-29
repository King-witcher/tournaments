import { SelectItem } from "@carbon/react";
import { useTranslations } from "next-intl";

import { Select } from "@/shared/ui/select";
import type { FieldErrors } from "../lib/form-errors";
import {
  type NoRepeatChoice,
  noRepeatChoices,
  type TournamentValues,
} from "../lib/form-values";

const BANS = [0, 1, 2, 3];

type Props = {
  values: TournamentValues;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** O torneio já começou: tudo aqui fica travado. */
  disabled: boolean;
};

/** Bans por mapa e a regra de não repetir campeão. */
export function TournamentChampionFields({
  values,
  onChange,
  errors,
  disabled,
}: Props) {
  const t = useTranslations("TournamentForm");

  return (
    <>
      <Select
        labelText={t("bansPerGame")}
        name="bansPerGame"
        value={values.bansPerGame}
        disabled={disabled}
        invalid={errors.bansPerGame !== undefined}
        invalidText={errors.bansPerGame}
        onChange={(e) => onChange({ bansPerGame: Number(e.target.value) })}
      >
        {BANS.map((n) => (
          <SelectItem key={n} value={n} text={t("bansOption", { count: n })} />
        ))}
      </Select>
      <Select
        labelText={t("noRepeat")}
        helperText={t(`noRepeatHints.${values.noRepeat}`)}
        name="noRepeat"
        value={values.noRepeat}
        disabled={disabled}
        invalid={errors.noRepeat !== undefined}
        invalidText={errors.noRepeat}
        onChange={(e) =>
          onChange({ noRepeat: e.target.value as NoRepeatChoice })
        }
      >
        {noRepeatChoices(values.noRepeat).map((choice) => (
          <SelectItem
            key={choice}
            value={choice}
            text={t(`noRepeatOptions.${choice}`)}
          />
        ))}
      </Select>
    </>
  );
}
