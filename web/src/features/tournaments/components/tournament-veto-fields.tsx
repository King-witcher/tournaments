import { FormGroup, SelectItem } from "@carbon/react";
import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Checkbox } from "@/shared/ui/checkbox";
import { ErrorText } from "@/shared/ui/error-text";
import { Select } from "@/shared/ui/select";
import type { FieldErrors } from "../lib/form-errors";
import {
  longestSeries,
  poolOf,
  type TournamentValues,
} from "../lib/form-values";
import styles from "./tournament-veto-fields.module.scss";

type Schemas = components["schemas"];

const MODES: Schemas["MapMode"][] = ["veto", "vote", "random", "off"];
const SECONDS = [15, 20, 30, 45, 60, 90, 120];

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
 * O veto das partidas: como sai o mapa de cada jogo, o pool de mapas (fora
 * do modo desligado; vale para o veto, a votação e o sorteio), o ban/pick
 * de campeões e o tempo de cada passo.
 */
export function TournamentVetoFields(props: Props) {
  const { values, onChange, errors, maps, disabled } = props;
  const t = useTranslations("TournamentForm.veto");
  const pool = poolOf(values, maps);
  const seconds = SECONDS.includes(values.vetoSeconds)
    ? SECONDS
    : [...SECONDS, values.vetoSeconds].sort((a, b) => a - b);
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
        helperText={t(`mapModeHint.${values.mapMode}`)}
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
      <Checkbox
        labelText={t("championPhase")}
        helperText={t("championPhaseHint")}
        name="championPhase"
        checked={values.championPhase}
        disabled={disabled}
        onChange={(championPhase) => onChange({ championPhase })}
      />
      {(values.mapMode !== "off" || values.championPhase) && (
        <Select
          labelText={t("seconds")}
          helperText={t("secondsHint")}
          name="vetoSeconds"
          value={values.vetoSeconds}
          disabled={disabled}
          invalid={errors.vetoSeconds !== undefined}
          invalidText={errors.vetoSeconds}
          onChange={(e) => onChange({ vetoSeconds: Number(e.target.value) })}
        >
          {seconds.map((s) => (
            <SelectItem
              key={s}
              value={s}
              text={t("secondsOption", { seconds: s })}
            />
          ))}
        </Select>
      )}
    </>
  );
}
