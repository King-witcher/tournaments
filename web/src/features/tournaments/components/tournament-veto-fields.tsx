import { FormGroup, SelectItem } from "@carbon/react";
import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { ErrorText } from "@/shared/ui/error-text";
import { Select } from "@/shared/ui/select";
import type { FieldErrors } from "../lib/form-errors";
import {
  isTdm,
  longestSeries,
  poolOf,
  STEP_SECONDS,
  stepSecondsGroup,
  type TournamentValues,
} from "../lib/form-values";
import { MapPoolOption } from "./map-pool-option";
import { SecondsSelect } from "./seconds-select";
import styles from "./tournament-veto-fields.module.scss";

type Schemas = components["schemas"];

const MODES: Schemas["MapMode"][] = ["veto", "vote", "random", "off"];

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
 * O grupo Mapas: como sai o mapa de cada partida, o pool de mapas com a
 * imagem de cada um (fora do modo desligado; vale para o veto, a votação e o
 * sorteio) e, com veto ou votação, o tempo de cada passo, que vale também
 * para o draft de campeões. Sem passo de mapa, esse tempo fica no grupo
 * Campeões.
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
              <MapPoolOption
                key={m.slug}
                map={m}
                // O foco do erro vai para a primeira caixa.
                name={i === 0 ? "mapPool" : undefined}
                checked={pool.includes(m.slug)}
                disabled={disabled}
                onChange={(on) => toggle(m.slug, on)}
              />
            ))}
          </div>
          {errors.mapPool && <ErrorText>{errors.mapPool}</ErrorText>}
        </FormGroup>
      )}
      {stepSecondsGroup(values) === "maps" && (
        <SecondsSelect
          labelText={t("seconds")}
          helperText={t(
            values.championPhase ? "secondsHintDraft" : "secondsHint",
          )}
          name="vetoSeconds"
          options={STEP_SECONDS}
          value={values.vetoSeconds}
          error={errors.vetoSeconds}
          disabled={disabled}
          onChange={(vetoSeconds) => onChange({ vetoSeconds })}
        />
      )}
    </>
  );
}
