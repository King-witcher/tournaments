import { SelectItem } from "@carbon/react";
import { useTranslations } from "next-intl";

import { Checkbox } from "@/shared/ui/checkbox";
import { RadioGroup } from "@/shared/ui/radio-group";
import { Select } from "@/shared/ui/select";
import type { FieldErrors } from "../lib/form-errors";
import type { TournamentValues } from "../lib/form-values";
import styles from "./tournament-format-fields.module.scss";

const BEST_OF = [1, 3, 5, 7, 9];

type Props = {
  values: TournamentValues;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** O torneio já começou: tudo aqui fica travado. */
  disabled: boolean;
};

/** A chave (as duas opções à vista), o reset da grande final e as séries. */
export function TournamentFormatFields({
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
    <>
      <RadioGroup
        legendText={t("bracket")}
        name="bracket"
        orientation="horizontal"
        value={values.bracket}
        disabled={disabled}
        options={[
          { value: "double", label: t("brackets.double") },
          { value: "single", label: t("brackets.single") },
        ]}
        onChange={(bracket) => onChange({ bracket })}
      />
      {values.bracket === "double" && (
        <Checkbox
          labelText={t("grandFinalReset")}
          helperText={t("grandFinalResetHint")}
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
    </>
  );
}
