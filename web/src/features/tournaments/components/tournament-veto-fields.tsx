import { FormGroup, SelectItem } from "@carbon/react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { Checkbox } from "@/shared/ui/checkbox";
import { ErrorText } from "@/shared/ui/error-text";
import { Select } from "@/shared/ui/select";
import { useFormatSeconds } from "../hooks/use-format-seconds";
import type { FieldErrors } from "../lib/form-errors";
import {
  isTdm,
  longestSeries,
  poolOf,
  type TournamentValues,
} from "../lib/form-values";
import styles from "./tournament-veto-fields.module.scss";

type Schemas = components["schemas"];

const MODES: Schemas["MapMode"][] = ["veto", "vote", "random", "off"];
const SECONDS = [15, 20, 30, 45, 60, 90, 120];
/** A janela da escolha simultânea do TDM (a API aceita de 15 a 300). */
const TEAM_PICK_SECONDS = [15, 30, 45, 60, 90, 120, 180, 300];

type Props = {
  values: TournamentValues;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** Os mapas ativos (`GET /maps`), para escolher o pool. */
  maps: Schemas["Map"][];
  /** O torneio já começou: tudo aqui fica travado. */
  disabled: boolean;
};

/**
 * O veto das séries: como sai o mapa de cada partida, o pool de mapas (fora
 * do modo desligado; vale para o veto, a votação e o sorteio) e o tempo de
 * cada passo. No TDM, com o draft de campeões no site, também o tempo da
 * janela simultânea. Se o draft é no site fica no grupo Campeões.
 */
export function TournamentVetoFields(props: Props) {
  const { values, onChange, errors, maps, disabled } = props;
  const t = useTranslations("TournamentForm.veto");
  const tdm = isTdm(values);
  const pool = poolOf(values, maps);
  const toggle = (slug: string, on: boolean) =>
    onChange({
      mapPool: on
        ? maps
            .filter((m) => pool.includes(m.slug) || m.slug === slug)
            .map((m) => m.slug)
        : pool.filter((s) => s !== slug),
    });

  return (
    <>
      <Select
        labelText={t("mapMode")}
        helperText={t(
          tdm
            ? `mapModeHintTdm.${values.mapMode}`
            : `mapModeHint.${values.mapMode}`,
        )}
        name="mapMode"
        value={values.mapMode}
        disabled={disabled}
        onChange={(e) =>
          onChange({ mapMode: e.target.value as Schemas["MapMode"] })
        }
      >
        {MODES.map((m) => (
          <SelectItem key={m} value={m} text={t(`mapModes.${m}`)} />
        ))}
      </Select>
      {values.mapMode !== "off" && (
        <FormGroup
          legendText={t("mapPool")}
          invalid={errors.mapPool !== undefined}
        >
          <p className={styles.hint}>
            {t("mapPoolHint", {
              count: pool.length,
              min: longestSeries(values),
            })}
          </p>
          <div className={styles.pool}>
            {maps.map((m, i) => (
              <Checkbox
                key={m.slug}
                // O foco do erro vai para a primeira caixa.
                name={i === 0 ? "mapPool" : undefined}
                labelText={m.name}
                checked={pool.includes(m.slug)}
                disabled={disabled}
                onChange={(on) => toggle(m.slug, on)}
              />
            ))}
          </div>
          {errors.mapPool && <ErrorText>{errors.mapPool}</ErrorText>}
        </FormGroup>
      )}
      {(values.mapMode !== "off" || values.championPhase) && (
        <SecondsSelect
          labelText={t("seconds")}
          helperText={t("secondsHint")}
          name="vetoSeconds"
          options={SECONDS}
          value={values.vetoSeconds}
          error={errors.vetoSeconds}
          disabled={disabled}
          onChange={(vetoSeconds) => onChange({ vetoSeconds })}
        />
      )}
      {tdm && values.championPhase && (
        <SecondsSelect
          labelText={t("teamPickSeconds")}
          helperText={t("teamPickSecondsHint")}
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
}

type SecondsProps = {
  labelText: ReactNode;
  helperText: ReactNode;
  name: string;
  options: number[];
  value: number;
  error?: string;
  disabled: boolean;
  onChange: (seconds: number) => void;
};

/**
 * Um tempo do veto, em segundos (em minutos a partir de um). Um valor que
 * veio da API fora das opções continua aparecendo.
 */
function SecondsSelect({
  options,
  value,
  error,
  onChange,
  ...props
}: SecondsProps) {
  const formatSeconds = useFormatSeconds();
  const all = options.includes(value)
    ? options
    : [...options, value].sort((a, b) => a - b);
  return (
    <Select
      {...props}
      value={value}
      invalid={error !== undefined}
      invalidText={error}
      onChange={(e) => onChange(Number(e.target.value))}
    >
      {all.map((s) => (
        <SelectItem key={s} value={s} text={formatSeconds(s)} />
      ))}
    </Select>
  );
}
