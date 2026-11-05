import { useTranslations } from "next-intl";
import { memo } from "react";

import type { components } from "@/shared/api/schema";
import { FormGroup } from "@/shared/ui/carbon/form";
import { SelectItem } from "@/shared/ui/carbon/select";
import { Checkbox } from "@/shared/ui/checkbox";
import { RadioGroup } from "@/shared/ui/radio-group";
import { Select } from "@/shared/ui/select";
import type { FieldErrors } from "../lib/form-errors";
import {
  hasPickWindow,
  isTdm,
  type NoRepeatChoice,
  noRepeatChoices,
  pickOrderChoices,
  STEP_SECONDS,
  stepSecondsGroup,
  TEAM_PICK_SECONDS,
  type TournamentValues,
} from "../lib/form-values";
import { sameValues } from "../lib/same-values";
import { SecondsSelect } from "./seconds-select";

const BANS = [0, 1, 2, 3];

const WHO: components["schemas"]["NoRepeatWho"][] = ["player", "team"];

/** Os campos do formulário que o bloco lê. */
const KEYS = [
  "championPhase",
  "bansPerSeries",
  "pickOrder",
  "mirror",
  "sameTeam",
  "noRepeat",
  "noRepeatWho",
  "mode",
  "mapMode",
  "vetoSeconds",
  "teamPickSeconds",
] as const;

type Props = {
  values: Pick<TournamentValues, (typeof KEYS)[number]>;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** O torneio já começou: tudo aqui fica travado. */
  disabled: boolean;
  /** As escolhas de não repetição, fora do torneio (a sala de veto só tem a da série). */
  repeatChoices?: NoRepeatChoice[];
};

/**
 * O draft de campeões da série: se é feito no site, os bans de cada lado, a ordem
 * das escolhas e o que pode repetir (na partida, entre adversários e, no TDM,
 * entre colegas; em outras partidas, pela regra de não repetição e, no TDM, quem
 * não repete). Com o draft no site: o tempo de cada passo, quando o mapa não
 * tem passo (com veto ou votação, ele fica no grupo Mapas), e, às cegas, o da
 * janela. Às cegas exige espelho: sem ele, a opção fica desabilitada, e desligar
 * o espelho com ela escolhida volta ao alternado.
 */
export const TournamentChampionFields = memo(function TournamentChampionFields({
  values,
  onChange,
  errors,
  disabled,
  repeatChoices,
}: Props) {
  const t = useTranslations("TournamentForm.championRules");
  const tVeto = useTranslations("TournamentForm.veto");
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
      {values.championPhase && (
        <Select
          labelText={t("pickOrder")}
          helperText={[
            t(`pickOrderHints.${values.pickOrder}`),
            !values.mirror && t("pickOrderBlindNeedsMirror"),
          ]
            .filter(Boolean)
            .join(" ")}
          name="pickOrder"
          value={values.pickOrder}
          disabled={disabled}
          invalid={errors.pickOrder !== undefined}
          invalidText={errors.pickOrder}
          onChange={(e) =>
            onChange({
              pickOrder: e.target.value as components["schemas"]["PickOrder"],
            })
          }
        >
          {pickOrderChoices(values.pickOrder).map((order) => (
            <SelectItem
              key={order}
              value={order}
              text={t(`pickOrderOptions.${order}`)}
              disabled={order === "blind" && !values.mirror}
            />
          ))}
        </Select>
      )}
      <FormGroup legendText={t("repeat")}>
        <Checkbox
          labelText={t("mirror")}
          helperText={values.mirror ? undefined : t("mirrorOff")}
          name="mirror"
          checked={values.mirror}
          disabled={disabled}
          onChange={(mirror) =>
            onChange(
              !mirror && values.pickOrder === "blind"
                ? { mirror, pickOrder: "alternating" }
                : { mirror },
            )
          }
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
        {(repeatChoices ?? noRepeatChoices(values.noRepeat)).map((choice) => (
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
      {stepSecondsGroup(values) === "champions" && (
        <SecondsSelect
          labelText={tVeto("seconds")}
          helperText={tVeto("secondsHint")}
          name="vetoSeconds"
          options={STEP_SECONDS}
          value={values.vetoSeconds}
          error={errors.vetoSeconds}
          disabled={disabled}
          onChange={(vetoSeconds) => onChange({ vetoSeconds })}
        />
      )}
      {hasPickWindow(values) && (
        <SecondsSelect
          labelText={tVeto("teamPickSeconds")}
          helperText={tVeto("teamPickSecondsHint")}
          name="teamPickSeconds"
          options={TEAM_PICK_SECONDS}
          value={values.teamPickSeconds}
          error={errors.teamPickSeconds}
          disabled={disabled}
          onChange={(teamPickSeconds) => onChange({ teamPickSeconds })}
        />
      )}
    </>
  );
}, sameValues(KEYS));
