import { useTranslations } from "next-intl";
import { memo } from "react";

import type { components } from "@/shared/api/schema";
import { SelectItem } from "@/shared/ui/carbon/select";
import { Select } from "@/shared/ui/select";
import type { FieldErrors } from "../lib/form-errors";
import {
  draftHasSteps,
  isTdm,
  STEP_SECONDS,
  stepSecondsGroup,
  type TournamentValues,
} from "../lib/form-values";
import { sameValues } from "../lib/same-values";
import { MapPoolField } from "./map-pool-field";
import { SecondsSelect } from "./seconds-select";
import { TournamentVetoOrderField } from "./tournament-veto-order-field";

type Schemas = components["schemas"];

const MODES: Schemas["MapMode"][] = ["veto", "vote", "random", "off"];

/** Os campos do formulário que o bloco lê. */
const KEYS = [
  "mapMode",
  "vetoOrder",
  "mode",
  "mapPool",
  "championPhase",
  "pickOrder",
  "bansPerSeries",
  "vetoSeconds",
  "stageFormat",
  "bestOf",
  "finalsBestOf",
] as const;

type Props = {
  /** A comunidade: o pool escolhido fica lembrado por comunidade. */
  tenant: string;
  values: Pick<TournamentValues, (typeof KEYS)[number]>;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** Os mapas ativos (`GET /maps`), para escolher o pool. */
  maps: Schemas["Map"][];
  /** O torneio já começou: tudo aqui fica travado. */
  disabled: boolean;
};

/**
 * O grupo Mapas: como sai o mapa de cada partida, no veto a ordem dos bans e
 * picks, o pool de mapas com a imagem de cada um (fora do modo desligado;
 * vale para o veto, a votação e o sorteio) e, com veto ou votação, o tempo de
 * cada passo, que vale também para o draft de campeões. Sem passo de mapa,
 * esse tempo fica no grupo Campeões.
 */
export const TournamentVetoFields = memo(function TournamentVetoFields(
  props: Props,
) {
  const { tenant, values, onChange, errors, maps, disabled } = props;
  const t = useTranslations("TournamentForm.veto");
  const tdm = isTdm(values);

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
      {values.mapMode === "veto" && (
        <TournamentVetoOrderField
          values={values}
          onChange={onChange}
          maps={maps}
          disabled={disabled}
        />
      )}
      {values.mapMode !== "off" && (
        <MapPoolField
          tenant={tenant}
          values={values}
          onChange={onChange}
          error={errors.mapPool}
          maps={maps}
          disabled={disabled}
        />
      )}
      {stepSecondsGroup(values) === "maps" && (
        <SecondsSelect
          labelText={t("seconds")}
          helperText={t(
            draftHasSteps(values) ? "secondsHintDraft" : "secondsHint",
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
}, sameValues(KEYS));
