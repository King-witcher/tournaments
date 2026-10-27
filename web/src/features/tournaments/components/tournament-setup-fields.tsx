import { useTranslations } from "next-intl";

import { Field } from "@/shared/ui/field";
import { Input } from "@/shared/ui/input";
import { Select } from "@/shared/ui/select";
import type { NoRepeatChoice, TournamentValues } from "../lib/form-values";

const BEST_OF = [1, 3, 5, 7, 9];
const BANS = [0, 1, 2, 3];
const NO_REPEAT: NoRepeatChoice[] = ["off", "match", "stage", "tournament"];

type Props = {
  values: TournamentValues;
  onChange: (patch: Partial<TournamentValues>) => void;
  /** O torneio já começou: tudo aqui fica travado. */
  disabled: boolean;
};

/** Vagas, chave, séries e regras de campeões. */
export function TournamentSetupFields({ values, onChange, disabled }: Props) {
  const t = useTranslations("TournamentForm");
  const bestOfOptions = BEST_OF.map((n) => (
    <option key={n} value={n}>
      {t("bestOfOption", { count: n })}
    </option>
  ));

  return (
    <fieldset disabled={disabled} className="flex flex-col gap-4">
      <legend className="mb-2 text-lg font-semibold">{t("setup")}</legend>
      <Field label={t("maxParticipants")} hint={t("maxParticipantsHint")}>
        <Input
          type="number"
          required
          min={2}
          max={1024}
          value={values.maxParticipants}
          onChange={(e) =>
            onChange({ maxParticipants: e.target.valueAsNumber })
          }
          className="w-32"
        />
      </Field>
      <Field label={t("bracket")}>
        <Select
          value={values.bracket}
          onChange={(e) =>
            onChange({ bracket: e.target.value as TournamentValues["bracket"] })
          }
        >
          <option value="double">{t("brackets.double")}</option>
          <option value="single">{t("brackets.single")}</option>
        </Select>
      </Field>
      {values.bracket === "double" && (
        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={values.grandFinalReset}
            onChange={(e) => onChange({ grandFinalReset: e.target.checked })}
          />
          {t("grandFinalReset")}
        </label>
      )}
      <div className="flex flex-wrap gap-4">
        <Field label={t("bestOf")}>
          <Select
            value={values.bestOf}
            onChange={(e) => onChange({ bestOf: Number(e.target.value) })}
          >
            {bestOfOptions}
          </Select>
        </Field>
        <Field label={t(`finalsBestOf.${values.bracket}`)}>
          <Select
            value={values.finalsBestOf}
            onChange={(e) => onChange({ finalsBestOf: Number(e.target.value) })}
          >
            {bestOfOptions}
          </Select>
        </Field>
        <Field label={t("bansPerGame")}>
          <Select
            value={values.bansPerGame}
            onChange={(e) => onChange({ bansPerGame: Number(e.target.value) })}
          >
            {BANS.map((n) => (
              <option key={n} value={n}>
                {t("bansOption", { count: n })}
              </option>
            ))}
          </Select>
        </Field>
      </div>
      <Field label={t("noRepeat")} hint={t("noRepeatHint")}>
        <Select
          value={values.noRepeat}
          onChange={(e) =>
            onChange({ noRepeat: e.target.value as NoRepeatChoice })
          }
        >
          {NO_REPEAT.map((choice) => (
            <option key={choice} value={choice}>
              {t(`noRepeatOptions.${choice}`)}
            </option>
          ))}
        </Select>
      </Field>
    </fieldset>
  );
}
