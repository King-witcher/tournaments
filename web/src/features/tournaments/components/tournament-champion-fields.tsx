import { SelectItem } from "@carbon/react";
import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Checkbox } from "@/shared/ui/checkbox";
import { RadioGroup } from "@/shared/ui/radio-group";
import { Select } from "@/shared/ui/select";
import type { FieldErrors } from "../lib/form-errors";
import {
  isTdm,
  type NoRepeatChoice,
  noRepeatChoices,
  type TournamentValues,
} from "../lib/form-values";

const BANS = [0, 1, 2, 3];

const WHO: components["schemas"]["NoRepeatWho"][] = ["player", "team"];

type Props = {
  values: TournamentValues;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** O torneio já começou: tudo aqui fica travado. */
  disabled: boolean;
};

/**
 * Bans por mapa e a regra de não repetir campeão. No TDM, também quem não
 * repete (só com a regra ligada) e se dois do mesmo time podem escolher o
 * mesmo campeão no mesmo mapa.
 */
export function TournamentChampionFields({
  values,
  onChange,
  errors,
  disabled,
}: Props) {
  const t = useTranslations("TournamentForm");
  const tdm = isTdm(values);

  return (
    <>
      <Select
        labelText={t("bansPerGame")}
        // No TDM, a dica diz quem bane (o capitão).
        helperText={
          tdm
            ? t("bansPerGameHintTdm", { count: values.bansPerGame })
            : undefined
        }
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
      {tdm && values.noRepeat !== "off" && (
        <RadioGroup
          legendText={t("noRepeatWho")}
          helperText={t(`noRepeatWhoHints.${values.noRepeatWho}`)}
          name="noRepeatWho"
          orientation="horizontal"
          value={values.noRepeatWho}
          disabled={disabled}
          options={WHO.map((value) => ({
            value,
            label: t(`noRepeatWhoOptions.${value}`),
          }))}
          onChange={(noRepeatWho) => onChange({ noRepeatWho })}
        />
      )}
      {tdm && (
        <Checkbox
          labelText={t("teamDuplicates")}
          helperText={t(
            values.teamDuplicates
              ? "teamDuplicatesHints.on"
              : "teamDuplicatesHints.off",
          )}
          name="teamDuplicates"
          checked={values.teamDuplicates}
          disabled={disabled}
          onChange={(teamDuplicates) => onChange({ teamDuplicates })}
        />
      )}
    </>
  );
}
