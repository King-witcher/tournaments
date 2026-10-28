import { SelectItem } from "@carbon/react";
import { useTranslations } from "next-intl";

import { Checkbox } from "@/shared/ui/checkbox";
import { Select } from "@/shared/ui/select";
import type { FieldErrors } from "../lib/form-errors";
import type { NoRepeatChoice, TournamentValues } from "../lib/form-values";
import { TournamentFormGroup } from "./tournament-form-group";
import styles from "./tournament-setup-fields.module.scss";

const BEST_OF = [1, 3, 5, 7, 9];
const BANS = [0, 1, 2, 3];
const NO_REPEAT: NoRepeatChoice[] = ["off", "match", "stage", "tournament"];

type Props = {
  values: TournamentValues;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** O torneio já começou: tudo aqui fica travado. */
  disabled: boolean;
};

/** Formato: chave, séries e regras de campeões. */
export function TournamentSetupFields({
  values,
  onChange,
  errors,
  disabled,
}: Props) {
  const t = useTranslations("TournamentForm");
  const bestOfItems = BEST_OF.map((n) => (
    <SelectItem key={n} value={n} text={t("bestOfOption", { count: n })} />
  ));

  return (
    <TournamentFormGroup legend={t("setup")}>
      <Select
        labelText={t("bracket")}
        name="bracket"
        value={values.bracket}
        disabled={disabled}
        onChange={(e) =>
          onChange({ bracket: e.target.value as TournamentValues["bracket"] })
        }
      >
        <SelectItem value="double" text={t("brackets.double")} />
        <SelectItem value="single" text={t("brackets.single")} />
      </Select>
      {values.bracket === "double" && (
        <Checkbox
          labelText={t("grandFinalReset")}
          name="grandFinalReset"
          checked={values.grandFinalReset}
          disabled={disabled}
          onChange={(grandFinalReset) => onChange({ grandFinalReset })}
        />
      )}
      <div className={styles.pair}>
        <Select
          labelText={t("bestOf")}
          name="bestOf"
          value={values.bestOf}
          disabled={disabled}
          invalid={errors.bestOf !== undefined}
          invalidText={errors.bestOf}
          onChange={(e) => onChange({ bestOf: Number(e.target.value) })}
        >
          {bestOfItems}
        </Select>
        <Select
          labelText={t(`finalsBestOf.${values.bracket}`)}
          name="finalsBestOf"
          value={values.finalsBestOf}
          disabled={disabled}
          onChange={(e) => onChange({ finalsBestOf: Number(e.target.value) })}
        >
          {bestOfItems}
        </Select>
      </div>
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
        helperText={t("noRepeatHint")}
        name="noRepeat"
        value={values.noRepeat}
        disabled={disabled}
        invalid={errors.noRepeat !== undefined}
        invalidText={errors.noRepeat}
        onChange={(e) =>
          onChange({ noRepeat: e.target.value as NoRepeatChoice })
        }
      >
        {NO_REPEAT.map((choice) => (
          <SelectItem
            key={choice}
            value={choice}
            text={t(`noRepeatOptions.${choice}`)}
          />
        ))}
      </Select>
    </TournamentFormGroup>
  );
}
