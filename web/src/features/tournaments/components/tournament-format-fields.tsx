import { useTranslations } from "next-intl";
import { memo } from "react";

import { SelectItem } from "@/shared/ui/carbon/select";
import { Checkbox } from "@/shared/ui/checkbox";
import { RadioGroup } from "@/shared/ui/radio-group";
import { Select } from "@/shared/ui/select";
import type { FieldErrors } from "../lib/form-errors";
import { isTdm, type TournamentValues, teamSizeOf } from "../lib/form-values";
import { isEvenSeries, mapsToWin, slotLimits, teamsIn } from "../lib/format";
import { sameValues } from "../lib/same-values";
import { DEFAULT_TIEBREAKERS } from "../lib/tiebreakers";
import styles from "./tournament-format-fields.module.scss";

/**
 * A eliminação, o suíço e a disputa de posições: só ímpares, porque a série não
 * pode empatar.
 */
const BEST_OF = [1, 3, 5, 7, 9];

/** O todos contra todos também aceita as séries pares. */
const BEST_OF_ROUND_ROBIN = [1, 2, 3, 4, 5, 6, 7, 8, 9];

/** O teto de rodadas do suíço na API. */
const MAX_SWISS_ROUNDS = 127;

/**
 * A escolha do formato numa opção só: as duas eliminações, round robin, suíço
 * e disputa de posições.
 */
type FormatChoice = "double" | "single" | "round_robin" | "swiss" | "placement";

const CHOICES: FormatChoice[] = [
  "double",
  "single",
  "round_robin",
  "swiss",
  "placement",
];

function choiceOf(
  values: Pick<TournamentValues, "stageFormat" | "bracket">,
): FormatChoice {
  return values.stageFormat === "elimination"
    ? values.bracket
    : values.stageFormat;
}

/**
 * Trocar de formato. Fora do todos contra todos, a série par vira a ímpar logo
 * acima (md4 vira md5), e o campo mostra o valor novo. Numa fase de tabela,
 * os desempates voltam ao padrão dela (o suíço começa pelo Buchholz).
 */
function patchOf(
  choice: FormatChoice,
  bestOf: number,
): Partial<TournamentValues> {
  if (choice === "round_robin")
    return { stageFormat: choice, tiebreakers: DEFAULT_TIEBREAKERS[choice] };
  const series = isEvenSeries(bestOf) ? { bestOf: bestOf + 1 } : {};
  if (choice === "placement") return { stageFormat: choice, ...series };
  return choice === "double" || choice === "single"
    ? { stageFormat: "elimination", bracket: choice, ...series }
    : {
        stageFormat: choice,
        tiebreakers: DEFAULT_TIEBREAKERS[choice],
        ...series,
      };
}

/** Os campos do formulário que o bloco lê. */
const KEYS = [
  "stageFormat",
  "bracket",
  "grandFinalReset",
  "bestOf",
  "finalsBestOf",
  "swissRounds",
  "mode",
  "teamSize",
  "maxParticipants",
  "countsForRating",
] as const;

type Props = {
  values: Pick<TournamentValues, (typeof KEYS)[number]>;
  onChange: (patch: Partial<TournamentValues>) => void;
  errors: FieldErrors;
  /** O torneio já começou: tudo aqui fica travado. */
  disabled: boolean;
};

/**
 * O formato (as cinco opções à vista, com a dica do escolhido), o reset da
 * grande final na dupla, as séries (e a final, na eliminação; as pares só no
 * todos contra todos, com a dica) e as rodadas do suíço. No TDM, o teto do
 * formato aparece em jogadores (como as vagas) e as rodadas contam times.
 */
export const TournamentFormatFields = memo(function TournamentFormatFields({
  values,
  onChange,
  errors,
  disabled,
}: Props) {
  const t = useTranslations("TournamentForm");
  const choice = choiceOf(values);
  const elimination = values.stageFormat === "elimination";
  const items = (options: number[]) =>
    options.map((n) => (
      <SelectItem key={n} value={n} text={t("bestOfOption", { count: n })} />
    ));
  const roundRobin = values.stageFormat === "round_robin";
  const tdm = isTdm(values);
  const size = teamSizeOf(values);
  // Rodadas que fazem sentido com as vagas: no máximo N − 1, com N os lados
  // da chave (no TDM, os times que as vagas comportam).
  const { teams } = teamsIn(values.maxParticipants ?? 2, size);
  const roundsMax = Math.max(1, Math.min(MAX_SWISS_ROUNDS, teams - 1));
  const roundOptions = Array.from({ length: roundsMax }, (_, i) => i + 1);
  if (values.swissRounds !== null && values.swissRounds > roundsMax) {
    roundOptions.push(values.swissRounds);
  }

  return (
    <>
      <RadioGroup
        legendText={t("stageFormat")}
        helperText={t(`stageFormatHints.${choice}`, {
          max: slotLimits(values.stageFormat, size).max,
        })}
        name="stageFormat"
        orientation="horizontal"
        value={choice}
        disabled={disabled}
        options={CHOICES.map((value) => ({
          value,
          label: t(`stageFormats.${value}`),
        }))}
        onChange={(next) => onChange(patchOf(next, values.bestOf))}
      />
      {elimination && values.bracket === "double" && (
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
          helperText={
            isEvenSeries(values.bestOf)
              ? t("bestOfEvenHint", { wins: mapsToWin(values.bestOf) })
              : undefined
          }
          name="bestOf"
          value={values.bestOf}
          disabled={disabled}
          invalid={errors.bestOf !== undefined}
          invalidText={errors.bestOf}
          onChange={(e) => onChange({ bestOf: Number(e.target.value) })}
        >
          {items(roundRobin ? BEST_OF_ROUND_ROBIN : BEST_OF)}
        </Select>
        {elimination && (
          <Select
            labelText={t(`finalsBestOf.${values.bracket}`)}
            name="finalsBestOf"
            value={values.finalsBestOf}
            disabled={disabled}
            onChange={(e) => onChange({ finalsBestOf: Number(e.target.value) })}
          >
            {items(BEST_OF)}
          </Select>
        )}
        {values.stageFormat === "swiss" && (
          <Select
            labelText={t("swissRounds")}
            helperText={t(tdm ? "swissRoundsHintTdm" : "swissRoundsHint")}
            name="swissRounds"
            value={values.swissRounds ?? "auto"}
            disabled={disabled}
            invalid={errors.swissRounds !== undefined}
            invalidText={errors.swissRounds}
            onChange={(e) =>
              onChange({
                swissRounds:
                  e.target.value === "auto" ? null : Number(e.target.value),
              })
            }
          >
            <SelectItem
              value="auto"
              text={t(tdm ? "swissRoundsAutoTdm" : "swissRoundsAuto")}
            />
            {roundOptions.map((n) => (
              <SelectItem
                key={n}
                value={n}
                text={t("swissRoundsOption", { count: n })}
              />
            ))}
          </Select>
        )}
      </div>
      <Checkbox
        labelText={t("countsForRating")}
        helperText={t("countsForRatingHint")}
        name="countsForRating"
        checked={values.countsForRating}
        disabled={disabled}
        onChange={(countsForRating) => onChange({ countsForRating })}
      />
    </>
  );
}, sameValues(KEYS));
