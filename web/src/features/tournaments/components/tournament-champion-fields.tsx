import { FormGroup, SelectItem } from "@carbon/react";
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
 * O draft de campeões da série: se é feito no site, os bans de cada lado e o
 * que pode repetir (na partida, entre adversários e, no TDM, entre colegas;
 * em outras partidas, pela regra de não repetição e, no TDM, quem não
 * repete).
 */
export function TournamentChampionFields({
  values,
  onChange,
  errors,
  disabled,
}: Props) {
  const t = useTranslations("TournamentForm.championRules");
  const tdm = isTdm(values);

  return (
    <>
      <RadioGroup
        legendText={t("phase")}
        name="championPhase"
        orientation="horizontal"
        value={values.championPhase ? "site" : "off"}
        disabled={disabled}
        options={(["site", "off"] as const).map((value) => ({
          value,
          label: t(`phaseOptions.${value}`),
        }))}
        onChange={(v) => onChange({ championPhase: v === "site" })}
      />
      <Select
        labelText={t("bans")}
        helperText={t("bansHint")}
        name="bansPerSeries"
        value={values.bansPerSeries}
        disabled={disabled}
        invalid={errors.bansPerSeries !== undefined}
        invalidText={errors.bansPerSeries}
        onChange={(e) => onChange({ bansPerSeries: Number(e.target.value) })}
      >
        {BANS.map((n) => (
          <SelectItem key={n} value={n} text={t("bansOption", { count: n })} />
        ))}
      </Select>
      <FormGroup legendText={t("repeat")}>
        <Checkbox
          labelText={t("mirror")}
          helperText={values.mirror ? undefined : t("mirrorOff")}
          name="mirror"
          checked={values.mirror}
          disabled={disabled}
          onChange={(mirror) => onChange({ mirror })}
        />
        {tdm && (
          <Checkbox
            labelText={t("sameTeam")}
            name="sameTeam"
            checked={values.sameTeam}
            disabled={disabled}
            onChange={(sameTeam) => onChange({ sameTeam })}
          />
        )}
      </FormGroup>
      <Select
        labelText={t("otherGames")}
        // Na série, a opção se explica; fora dela, conta só o que foi jogado
        // e o pool pode não bastar.
        helperText={
          values.noRepeat === "stage" || values.noRepeat === "tournament"
            ? t(`otherGamesHints.${values.noRepeat}`)
            : undefined
        }
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
            text={t(`otherGamesOptions.${choice}`)}
          />
        ))}
      </Select>
      {tdm && values.noRepeat !== "off" && (
        <RadioGroup
          legendText={t("who")}
          helperText={values.noRepeatWho === "team" ? t("whoTeam") : undefined}
          name="noRepeatWho"
          orientation="horizontal"
          value={values.noRepeatWho}
          disabled={disabled}
          options={WHO.map((value) => ({
            value,
            label: t(`whoOptions.${value}`),
          }))}
          onChange={(noRepeatWho) => onChange({ noRepeatWho })}
        />
      )}
    </>
  );
}
