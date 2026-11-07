import { useTranslations } from "next-intl";
import { memo } from "react";

import type { components } from "@/shared/api/schema";
import { FormRow } from "@/shared/ui/form-row";
import { RadioTileGroup } from "@/shared/ui/radio-tile-group";
import type { FieldErrors } from "../lib/form-errors";
import {
  draftHasSteps,
  isTdm,
  onlyEvenSeries,
  STEP_SECONDS,
  singleLength,
  stepSecondsGroup,
  type TournamentValues,
} from "../lib/form-values";
import { isEvenSeries } from "../lib/format";
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
  "stages",
] as const;

type Props = {
  /** A comunidade: o pool escolhido fica lembrado por comunidade. */
  tenant: string;
  values: Pick<TournamentValues, (typeof KEYS)[number]>;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** Os mapas ativos (`GET /maps`), para escolher o pool. */
  maps: Schemas["Map"][];
  /** O torneio já começou: tudo aqui só para leitura. */
  readOnly: boolean;
};

/**
 * O grupo Mapas: como sai o mapa de cada partida (em tiles, numa fileira,
 * cada modo com o que faz), no veto a ordem dos bans e picks (a série par só
 * tem bans, e o campo não aparece: o tile do veto explica), o pool de mapas
 * com a imagem de cada um (fora do modo desligado; vale para o veto, a
 * votação e o sorteio) e, com veto ou votação, o tempo de cada passo (um
 * campo curto, na primeira coluna), que vale também para o draft de campeões.
 * Sem passo de mapa, esse tempo fica no grupo Campeões.
 */
export const TournamentVetoFields = memo(function TournamentVetoFields(
  props: Props,
) {
  const { tenant, values, onChange, errors, maps, readOnly } = props;
  const t = useTranslations("TournamentForm.veto");
  const tdm = isTdm(values);
  const evenVeto = values.mapMode === "veto" && onlyEvenSeries(values);
  // A dica da série par conta as partidas de uma só duração; com várias, a geral.
  const evenGames = singleLength(values);

  return (
    <>
      <RadioTileGroup
        legend={t("mapMode")}
        name="mapMode"
        value={values.mapMode}
        readOnly={readOnly}
        columns={4}
        options={MODES.map((m) => ({
          value: m,
          content: t(`mapModes.${m}`),
          // A série par só tem bans: o veto dela se explica diferente.
          description:
            m === "veto" && evenGames !== undefined && isEvenSeries(evenGames)
              ? t(tdm ? "mapModeHintTdm.vetoEven" : "mapModeHint.vetoEven", {
                  games: evenGames,
                })
              : t(tdm ? `mapModeHintTdm.${m}` : `mapModeHint.${m}`),
        }))}
        onChange={(mapMode) => onChange({ mapMode })}
      />
      {values.mapMode === "veto" && !evenVeto && (
        <TournamentVetoOrderField
          values={values}
          onChange={onChange}
          maps={maps}
          readOnly={readOnly}
        />
      )}
      {values.mapMode !== "off" && (
        <MapPoolField
          tenant={tenant}
          values={values}
          onChange={onChange}
          error={errors.mapPool}
          maps={maps}
          readOnly={readOnly}
        />
      )}
      {stepSecondsGroup(values) === "maps" && (
        <FormRow columns={4}>
          <SecondsSelect
            labelText={t("seconds")}
            helperText={t(
              draftHasSteps(values) ? "secondsHintDraft" : "secondsHint",
            )}
            name="vetoSeconds"
            options={STEP_SECONDS}
            value={values.vetoSeconds}
            error={errors.vetoSeconds}
            readOnly={readOnly}
            onChange={(vetoSeconds) => onChange({ vetoSeconds })}
          />
        </FormRow>
      )}
    </>
  );
}, sameValues(KEYS));
